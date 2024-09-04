use std::path::{Path, PathBuf};
use std::{io, vec};
use std::error::Error;
use std::fs;
use log::error;
use colored::*;
use dirs::home_dir;
use std::io::Write;

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

// takes a vec of files and saves their paths to a file 'nextsyncengine-failed_uploads.txt' in the users home dir
fn save_failed_files_txt(failed_files: &Vec<File>) -> Result<(), Box<io::Error>>  {
    if let Some(home_dir) = home_dir() {
        // create a file and open it in append mode
        let mut failed_upload_txt = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(home_dir.join("nextsyncengine-failed_uploads.txt"))?;

        // go throught the vec of failed files and write all paths to the file
        for f in failed_files {
            failed_upload_txt.write_all(format!("{:?}\n", f.get_local_path()).as_bytes())?;
        }
        println!("{}", format!("You can find the paths of the files which failed to upload at {:?}", home_dir.join("nextsyncengine-failed_uploads.txt")).red());
        Ok(())
    } else {
        Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "Could not locate the users home directory!")))
    }

}

// splits a vector into two new vectors each containing one half of the original vector
fn split_vec_to_vecs(vec_org: Vec<File>, num_threads: usize) -> Vec<Vec<File>> {
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
                error!("Index out of bound when splitting vec_org");
                panic!()
            }
            results.push(vec_org[current_index..(current_index + len)].to_vec());
            current_index += len;
        }
    }

    results
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

// starts a upload batch with a fall back from which you can continue if some file uploads fail
pub fn start_upload(files: Vec<File>, client: NextcloudClient, num_threads: usize) -> Result<(), Box<dyn Error>> {
    let fallback_client = client.clone();
    match threaded_upload(files, client, num_threads) {
        // files uploaded without a fatal error
        Ok(failed_files) => {
            // check if any files weren't uploaded
            if failed_files.len() == 0 {
                return Ok(())
            }

            // ask the user if he wants to try uploading again
            println!("{}", format!("{} file(s) could not be uploaded:", failed_files.len()).red());
            for file in failed_files.iter() {
                println!("{}", format!("{:?}", file.get_local_path()).red());
            }
            print!("{}", format!("Try again?\nYes(y) or No(n) ").yellow());

            let mut answer = String::new();
            let _ = io::stdin().read_line(&mut answer);
            if answer.trim().to_lowercase() == "y" || answer.trim().to_lowercase() == "yes" {
                match threaded_upload(failed_files, fallback_client, num_threads) {
                    // files uploaded without a fatal error
                    Ok(second_failed_files) => {
                        // check if any files weren't uploaded
                        if second_failed_files.len() == 0 {
                            return Ok(())
                        }
                        // write those file paths to a file in the users home dir
                        println!("{}", "Second uploading attempt failed too!".red());
                        return Ok(save_failed_files_txt(&second_failed_files)?)
                    }
                    Err(e) => return Err(e)
                }
            }
            else {
                // write the file paths to a file in the users home dir if the upload failed on the first attempt
                return Ok(save_failed_files_txt(&failed_files)?)
            }
        }
        Err(e) => return Err(e)
    }    
}

// starts the uploads in 4 parallel threads
fn threaded_upload(files: Vec<File>, client: NextcloudClient, num_threads: usize) -> Result<Vec<File>, Box<io::Error>> {
    // calculate the totat upload size
    let mut total_size: u64 = 0;
    for file in &files {
        total_size += file.get_size()
    }

    // create a shared nextcloud_client and a counter to track the upload progress and update the progress bar accordingly 
    let shared_client = Arc::new(client);
    let shared_uploaded_size: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    let shared_failed_files: Arc<Mutex<Vec<File>>> = Arc::new(Mutex::new(vec![]));

    // split the original vec 'files' in 4 seperate vecs and pass each one of them to a seperate thread
    let splitted_files: Vec<Vec<File>> = split_vec_to_vecs(files, num_threads);

    // print initial progress bar
    update_progress_bar(0, total_size);

    // spawning the uploading threads
    let mut threads: Vec<JoinHandle<()>> = vec![];
    for v in splitted_files {
        let uploaded_size = Arc::clone(&shared_uploaded_size);
        let client_clone = Arc::clone(&shared_client);
        let failed_files_clone = Arc::clone(&shared_failed_files);
        threads.push(std::thread::spawn(move || {
        upload_files(v, client_clone, total_size, uploaded_size, failed_files_clone);
        }));
    }

    // joining the uploading threads
    for thread in threads {
        match thread.join() {
            Err(_) => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to join upload threads!"))),
            _ => {}
        };
    }
    let failed_files = shared_failed_files.lock().unwrap().to_owned();
    Ok(failed_files)
}

// uploads a vec of files to nextcloud and updates the progress bar
fn upload_files(files: Vec<File>, client: Arc<NextcloudClient>, total_size: u64, shared_uploaded_size: Arc<Mutex<u64>>, shared_failed_files: Arc<Mutex<Vec<File>>>) {
    for file in files {
        let size = file.get_size();
        // uplaoding the current file to nextcloud
        if let Err(e) = client.upload_file(&file) {
            // loading error and saving the file which could not be uploaded
            error!("{}", e);
            let mut failed_files = shared_failed_files.lock().unwrap();
            failed_files.push(file);
            continue
        }

        // updating the progress bar
        let mut uploaded_size = shared_uploaded_size.lock().unwrap();
        *uploaded_size += size;
        update_progress_bar(*uploaded_size, total_size);
    }
}
