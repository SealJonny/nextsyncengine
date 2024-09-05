use crate::nextcloud::NextcloudClient;
use crate::media::Extractor;
use crate::upload::common;
use crate::filesystem::File;

use std::path::{Path, PathBuf};
use std::error::Error;
use colored::*;


fn get_remote_parent(files: &mut Vec<File>, root_folder: PathBuf) {
    for file in files {
        file.set_remote_parent(root_folder.clone());
    }
}

// Todo: implement keeping the original structure
// uploads a folder to Nextcloud keeping the original structure
pub fn upload_unsorted(path_upload: String, from_folder: bool, remote_path: String, num_threads: usize, client: NextcloudClient, extractor: Extractor) -> Result<(), Box<dyn Error>> {
    // check if the root folder exists and if not ask the user if he wants to create it
    match common::exists_root_folder(Path::new(&remote_path), &client) {
        Ok(true) => {}
        Ok(false) => return Ok(()),
        Err(e) => return Err(e)
    }

    let root_folder = PathBuf::from(remote_path);

    print!("{}", "Scanning local folder for files ... ".green());
    // creating the missing folders on nextcloud and uploading the files in 4 threads to nextcloud
    match common::get_files_for_upload(Path::new(&path_upload), from_folder, &extractor) {
        Ok(mut files) => {
            println!("{}", "done".green());

            get_remote_parent(&mut files, root_folder);
            return common::start_upload(files, client, num_threads)
        }
        
        // passing error to caller function
        Err(e) => return Err(e)

    }
}