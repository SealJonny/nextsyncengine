use chrono::{Datelike, Local, TimeZone};
use chrono::offset::LocalResult;

use std::path::{Path, PathBuf};
use std::io;
use std::error::Error;
use colored::*;

use crate::nextcloud::NextcloudClient;
use crate::filesystem::{File, Folder};
use crate::media::Extractor;
use crate::upload::common;

// assigns each file a remote parent based on the mtime of the file
fn get_remote_parent(files: &mut Vec<File>, mut root: Folder, client: &NextcloudClient, depth: &str) -> Result<(), Box<dyn Error>> {
    // goes through the list of files and determines the remote parent of the file
    for file in files {
        if let LocalResult::Single(mtime) = Local.timestamp_opt(file.get_mtime(), 0) {
            let day: String;
            if mtime.day() < 10 {
                day = format!("0{}", mtime.day());
            } else  {
                day = format!("{}", mtime.day());
            }
    
            let month: String;
            if mtime.month() < 10 {
                month = format!("0{}", mtime.month());
    
            } else {
                month = format!("{}", mtime.month());
            }
            let year = format!("{}", mtime.year());
            let root_str = root.get_name();
            let root_path = Path::new(root_str.as_str());
    
            let year_path = root_path.join(Path::new(year.as_str()));
            
            if !root.has_subfolder(&year_path) {
                client.create_folder(&year_path)?;
                root.add_sub_folder(Folder::new(year), root_path);
            }
            if depth == "year" {
                file.set_remote_parent(year_path);
                continue
            }
    
            let month_path = year_path.join(Path::new(month.as_str()));
    
            if !root.has_subfolder(&month_path) {
                client.create_folder(&month_path)?;
                root.add_sub_folder(Folder::new(month), &year_path);
            }
            if depth == "month" {
                file.set_remote_parent(month_path);
                continue
            }
    
            let day_path = month_path.join(Path::new(day.as_str()));
            if !root.has_subfolder(&day_path) {
                client.create_folder(&day_path)?;
                root.add_sub_folder(Folder::new(day), &month_path);
            }
            file.set_remote_parent(day_path);

        } else {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to parse unix timestamp into a DateTime object!")))
        }
    }
    Ok(())
}

// travels through the remote folder and recursively adds all folders as sub folders to 'root'
fn travel_dir_dav(root: &mut Folder, client: &NextcloudClient) -> Result<(), Box<dyn Error>> {
    let mut paths_folder: Vec<PathBuf> = Vec::new();
    paths_folder.push(Path::new(root.get_name().as_str()).to_path_buf());

    //lists the sub folders in a folder and adds them to 'root' as sub folders and pushes them into 'paths_folder'
    while let Some(current_folder) = paths_folder.pop() {
        let sub_folders = client.ls(&current_folder)?;
        for sub_folder in sub_folders {
            if let Some(name) = sub_folder.to_str() {
                root.add_sub_folder(Folder::new(name.to_string()), &current_folder);
                paths_folder.push(current_folder.join(sub_folder));
            }
        }
    }
    Ok(())
}

pub fn upload_sorted(local_path: String, remote_path: String, depth: String, num_threads: usize, client: NextcloudClient, extractor: Extractor) -> Result<(), Box<dyn Error>> {
    // check if the root folder exists and if not ask the user if he wants to create it
    match common::exists_root_folder(Path::new(&remote_path), &client) {
        Ok(true) => {}
        Ok(false) => return Ok(()),
        Err(e) => return Err(e)
    }

    // create the cached version of the nextcloud folder structure
    let mut root = Folder::new(remote_path.to_owned());
    if let Err(e)= travel_dir_dav(&mut root, &client) {
        return Err(e)
    }

    print!("{}", "Scanning local folder for files ... ".green());
    // creating the missing folders on nextcloud and uploading the files in 4 threads to nextcloud
    match common::trave_dir_local(Path::new(&local_path), &extractor) {
        Ok(mut files) => {
            println!("{}", "done".green());
            
            print!("{}", "Creating the folder structure on Nextcloud ... ".green());
            get_remote_parent(&mut files, root, &client, &depth)?;
            println!("{}", "done".green());

            common::threaded_upload(files, client, num_threads)?;
            Ok(())
        }
        
        // passing error to caller function
        Err(e) => return Err(e)

    }
}