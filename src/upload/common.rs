use std::path::{Path, PathBuf};
use std::{io, vec};
use std::error::Error;
use std::fs;
use log::error;
use colored::*;

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::nextcloud::NextcloudClient;
use crate::filesystem::File;
use crate::media::Extractor;
use crate::helpers;

// updates the terminal progress bar using the helpers::progress_bar function
fn update_progress_bar(uploaded_size: u64, total_size: u64) {
    // prettify the progress counter through converting the numbers into the suitable unit
    let unit: String;
    let mut uploaded_size_rounded = uploaded_size as f64;
    let mut total_size_rounded = total_size as f64;
    if total_size_rounded >= 1_000_000_000.0 {
        total_size_rounded /= 1_000_000_000.0;
        uploaded_size_rounded /= 1_000_000_000.0;
        unit = "G".to_string();
    } else {
        total_size_rounded /= 1_000_000.0;
        uploaded_size_rounded /= 1_000_000.0;
        unit = "M".to_string();
    }

    // build the suffix and print the updated progress bar using helpers::progress_bar
    let suffix = format!("{:.2}{}/{:.2}{}", uploaded_size_rounded, &unit, total_size_rounded, &unit);
    helpers::progress_bar(uploaded_size, total_size, "Uploading", &suffix)
}

// travels through the local folder and recursively stores all files in a vector
pub fn trave_dir_local(root_path: &Path, extractor: &Extractor) -> Result<Vec<File>, Box<dyn Error>> {
    let mut paths_folder: Vec<PathBuf> = Vec::new();
    paths_folder.push(root_path.to_path_buf());

    let mut files: Vec<File> = Vec::new();

    // lists the items in a folder and add the subfolders to 'paths_folder' and the files to 'files'
    while let Some(current_folder) = paths_folder.pop() {
        let entries = fs::read_dir(current_folder)?;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
    
            if file_type.is_dir() {
                paths_folder.push(entry.path());
                continue
            }
            let mtime = extractor.extract_date_time(entry.path().as_path())?;
            files.push(File::new(entry.path().as_path(), mtime));
        }
    }
    Ok(files)
}

// uploads a vec of files to nextcloud and updates the progress bar
pub fn upload_files(files: Vec<File>, client: Arc<NextcloudClient>, total_size: u64, shared_uploaded_size: Arc<Mutex<u64>>) {
    for file in files {
        let size = file.get_size();
        // uplaoding the current file to nextcloud
        if let Err(e) = client.upload_file(file) {
            error!("{}", e);
        } else {
            // updating the progress bar
            let mut uploaded_size = shared_uploaded_size.lock().unwrap();
            *uploaded_size += size;
            update_progress_bar(*uploaded_size, total_size);
        }
    }
}

// // splits a vector into two new vectors each containing one half of the original vector
// pub fn split_vec_to_vecs(vec_org: Vec<File>) -> (Vec<File>, Vec<File>) {
//     let mid: usize;
//     // determine the mid index of the vec
//     if vec_org.len() % 2 == 0 {
//         mid = vec_org.len() / 2;
//     } else {
//         mid = (vec_org.len() + 1) / 2;
//     }
//     (vec_org[..mid].to_vec(), vec_org[mid..].to_vec())
// }

// splits a vector into two new vectors each containing one half of the original vector
pub fn split_vec_to_vecs(vec_org: Vec<File>, num_threads: usize) -> Vec<Vec<File>> {
    // calculating the leftover of the division
    let mut leftover = vec_org.len() % num_threads;
    let mut splitted_vec_lens = vec![vec_org.len() / num_threads; num_threads];

    // distributing leftover evenly on the lenghts of the sub vectors
    while leftover > 0 {
        for iter in splitted_vec_lens.iter_mut() {
            if leftover == 0 {
                break
            }
            *iter += 1;
            leftover -= 1;
        }
    }

    // extracting the sub vectors based on the before calculated lenghts in splitted_vec_lens
    let mut current_index: usize = 0;
    let mut results: Vec<Vec<File>> = vec![];
    for i in 0..num_threads {
        if let Some(&len) = splitted_vec_lens.get(i) {
            if current_index + len > vec_org.len() {
                panic!("Index out of bound when splitting vec_org")
            }
            results.push(vec_org[current_index..(current_index + len)].to_vec());
            current_index += len;
        }
    }

    results
}

pub fn exists_root_folder(root_folder: &Path, client: &NextcloudClient) -> Result<bool, Box<dyn Error>> {
    // check if the root folder exists and if not ask the user if he wants to create it
    match client.exists_folder(root_folder) {
        Ok(val) => {
            if !val {
                print!("{}", format!("The folder {} does not exist on your Nextcloud instance.\nWould you like to create it?\nYes(y) or No(n) ", root_folder.to_str().unwrap_or_default()).yellow());
                let mut answer = String::new();
                let _ = io::stdin().read_line(&mut answer);
                if answer.trim().to_lowercase() == "y" || answer.trim().to_lowercase() == "yes" {
                    if let Err(e) = client.create_folder(root_folder) {
                        error!("{}", e);
                        return Err(e)
                    }
                } else {
                    return Ok(false)
                }
            }
            Ok(true)
        }
        Err(e) => return Err(e)
    }
}

// starts the uploads in 4 parallel threads
pub fn threaded_upload(files: Vec<File>, client: NextcloudClient, num_threads: usize) -> Result<(), Box<io::Error>> {
    // calculate the totat upload size
    let mut total_size: u64 = 0;
    for file in &files {
        total_size += file.get_size()
    }

    // create a shared nextcloud_client and a counter to track the upload progress and update the progress bar accordingly 
    let shared_client = Arc::new(client);
    let shared_uploaded_size: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    // split the original vec 'files' in 4 seperate vecs and pass each one of them to a seperate thread
    let splitted_files: Vec<Vec<File>> = split_vec_to_vecs(files, num_threads);

    // print initial progress bar
    update_progress_bar(0, total_size);

    // spawning the uploading threads
    let mut threads: Vec<JoinHandle<()>> = vec![];
    for v in splitted_files {
        let uploaded_size = Arc::clone(&shared_uploaded_size);
        let client_clone = Arc::clone(&shared_client);
        threads.push(std::thread::spawn(move || {
        upload_files(v, client_clone, total_size, uploaded_size);
        }));
    }

    // joining the uploading threads
    for thread in threads {
        match thread.join() {
            Err(_) => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to join upload threads!"))),
            _ => {}
        };
    }
    Ok(())
}

