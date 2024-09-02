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
pub fn upload_unsorted(local_path: String, remote_path: String, num_threads: usize, client: NextcloudClient, extractor: Extractor) -> Result<(), Box<dyn Error>> {
    // check if the root folder exists and if not ask the user if he wants to create it
    match common::exists_root_folder(Path::new(&remote_path), &client) {
        Ok(true) => {}
        Ok(false) => return Ok(()),
        Err(e) => return Err(e)
    }

    let root_folder = PathBuf::from(remote_path);

    print!("{}", "Scanning local folder for files ... ".green());
    // creating the missing folders on nextcloud and uploading the files in 4 threads to nextcloud
    match common::trave_dir_local(Path::new(&local_path), &extractor) {
        Ok(mut files) => {
            println!("{}", "done".green());

            get_remote_parent(&mut files, root_folder);
            common::threaded_upload(files, client, num_threads)?;
            Ok(())
        }
        
        // passing error to caller function
        Err(e) => return Err(e)

    }
}