mod nextcloud;
mod media;
mod filesystem;
mod helpers;

use chrono::offset::LocalResult;
use filesystem::{File, Folder};
use nextcloud::NextcloudClient;
use std::error::Error;
use std::thread::JoinHandle;
use std::{fs, io, vec};
use std::{env, path::PathBuf};
use std::path::Path;
use media::Extractor;
use flexi_logger::{Logger, Duplicate, FileSpec, WriteMode};
use log::error;
use chrono::{Datelike, Local, TimeZone};
use clap::{Arg, Command};
use std::sync::{Arc, Mutex};


fn init_logger(config_folder: &Path) {
    let log_filename = config_folder.join("process.log");

    // Initialize the logger
    Logger::try_with_str("warn")
        .unwrap()
        .log_to_file(
            FileSpec::try_from(log_filename).unwrap(),
        )
        .write_mode(WriteMode::BufferAndFlush) // Ensure logs are flushed to disk regularly
        .duplicate_to_stdout(Duplicate::Warn) // Optional: also output warnings and above to stdout
        .format_for_files(flexi_logger::detailed_format) // Format similar to Python's logging
        .start()
        .unwrap();
}

// returns the path to the parent folder on Nextcloud based on the mtime of the file
fn get_remote_parent(root: &mut Folder, client: &NextcloudClient, mtime: i64, depth: &str) -> Result<PathBuf, Box<dyn Error>> {
    if let LocalResult::Single(mtime) = Local.timestamp_opt(mtime, 0) {
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
            return Ok(year_path)
        }

        let month_path = year_path.join(Path::new(month.as_str()));

        if !root.has_subfolder(&month_path) {
            client.create_folder(&month_path)?;
            root.add_sub_folder(Folder::new(month), &year_path);
        }
        if depth == "month" {
            return Ok(month_path)
        }

        let day_path = month_path.join(Path::new(day.as_str()));
        if !root.has_subfolder(&day_path) {
            client.create_folder(&day_path)?;
            root.add_sub_folder(Folder::new(day), &month_path);
        }
        return Ok(day_path)

    }
    Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to parse unix timestamp into a DateTime object!")))
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

// travels through the local folder and recursively stores all files in a vector
fn trave_dir_local(root_path: &Path, root_folder: &mut Folder, client: &NextcloudClient, extractor: &Extractor, depth: String) -> Result<Vec<File>, Box<dyn Error>> {
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
            let remote_parent = get_remote_parent(root_folder, client, mtime, depth.as_str())?;
            files.push(File::new(entry.path().as_path(), &remote_parent, mtime));
        }
    }

    Ok(files)
}

// fn upload_folder(files: Vec<File>, client: &NextcloudClient) {
//     let mut total_size: u64 = 0;
//     for file in &files {
//         total_size += file.get_size()
//     }

//     let mut uploaded_size: u64 = 0;
//     helpers::progress_bar(uploaded_size, total_size, "Uploading", "");
//     for file in files {
//         let size = file.get_size();
//         if let Err(e) = client.upload_file(file) {
//             error!("{}", e);
//         } else {
//             uploaded_size += size;
//             helpers::progress_bar(uploaded_size, total_size, "Uploading", "");
//         }
//     }
// }

// uploads a vec of files to nextcloud and updates the progress bar
fn upload_folder(files: Vec<File>, client: Arc<NextcloudClient>, total_size: u64, shared_uploaded_size: Arc<Mutex<u64>>) {
    for file in files {
        let size = file.get_size();
        // uplaoding the current file to nextcloud
        if let Err(e) = client.upload_file(file) {
            error!("{}", e);
        } else {
            // updating the progress bar
            let mut uploaded_size = shared_uploaded_size.lock().unwrap();
            *uploaded_size += size;
            helpers::progress_bar(*uploaded_size, total_size, "Uploading", "");
        }
    }
}

// splits a vector into two new vectors each containing one half of the original vector
fn split_vec_to_vecs(vec_org: Vec<File>) -> (Vec<File>, Vec<File>) {
    let mid: usize;
    // determine the mid index of the vec
    if vec_org.len() % 2 == 0 {
        mid = vec_org.len() / 2;
    } else {
        mid = (vec_org.len() + 1) / 2;
    }
    (vec_org[..mid].to_vec(), vec_org[mid..].to_vec())
}

fn main() {
    let version = "0.1.0";

    let config_folder = Path::new("_nextsyncengine_");
    init_logger(config_folder);
    let path = config_folder.join(".env");
    dotenv::from_path(path).expect("Failed to read .env file");

    // Helper function to retrieve environment variables
    fn get_env_var(var_name: &str) -> String {
        env::var(var_name).unwrap_or_else(|e| {
            eprintln!("Error while reading '{}': {}", var_name, e);
            String::new()
        })
    }

    let server_url = get_env_var("SERVER_URL");
    let username = get_env_var("NC_USERNAME");
    let password = get_env_var("PASSWORD");
    let exiftool = get_env_var("EXIFTOOL");

    let client = NextcloudClient::new(server_url, username.clone(), password);
    let extractor = Extractor::new(exiftool);

    let mut release_mode = true;
    #[cfg(debug_assertions)]
    {
        release_mode = false;
    }

    // parser for cli options
    let matches = Command::new("nextsyncengine")
        .version(version)
        .about("Have a look at the README.md at https://github.com/SealJonny/nextsyncengine")
        .arg(
            Arg::new("local_path")
                .long("local_path")
                .value_parser(clap::value_parser!(String))
                .required(release_mode)
                .help("The path to your local folder which will be uploaded"),
        )
        .arg(
            Arg::new("remote_path")
                .long("remote_path")
                .value_parser(clap::value_parser!(String))
                .required(release_mode)
                .help("The location where on your Nextcloud server the folder will be uploaded"),
        )
        .arg(
            Arg::new("depth")
                .long("depth")
                .value_parser(clap::value_parser!(String))
                .default_value("month")
                .help("Options are: year, month, and day. Determines the depth of the folder structure."),
        )
        .get_matches();
    
    let local_path: String;
    let remote_path: String;
    let depth: String;

    #[cfg(not(debug_assertions))]
    {
        local_path = matches.get_one::<String>("local_path").expect("required").to_string();
        remote_path = matches.get_one::<String>("remote_path").expect("required").to_string();
        let default_depth = &"month".to_string();
        depth = matches.get_one::<String>("depth").unwrap_or(default_depth).to_string();
    }

    #[cfg(debug_assertions)]
    {
        local_path = "/home/sealjonny/Downloads/Converted".to_string();
        remote_path = "/TestDebug".to_string();
        depth = "month".to_string();
    }

    // checking if nextcloud server is online and not in maintenance mode and terminating execution if it is offline.
    print!("Checking if Nextcloud server is online ... ");
    match client.is_online() {
        Ok(val) => {
            if !val {
                println!("\nNextcloud server is offline or in maintenance mode!");
                return
            }
            println!("done")
        }
        Err(e) => {
            error!("{}", e);
            return
        }
    }

    println!("You are logged in as {}.", &username);

    // check if the root folder exists and if not ask the user if he wants to create it
    match client.exists_folder(Path::new(&remote_path)) {
        Ok(val) => {
            if !val {
                print!("The folder {} does not exist on your Nextcloud instance.\nWould you like to create it?\nYes(y) or No(n) ", &remote_path);
                let mut answer = String::new();
                let _ = io::stdin().read_line(&mut answer);
                if answer.trim().to_lowercase() == "y" || answer.trim().to_lowercase() == "yes" {
                    if let Err(e) = client.create_folder(Path::new(&remote_path)) {
                        error!("{}", e);
                    }
                } else {
                    return
                }
            }
        }
        Err(e) => error!("{}", e)
    }

    // create the cached version of the nextcloud folder structure
    let mut root = Folder::new(remote_path.to_owned());
    let _ = travel_dir_dav(&mut root, &client);

    print!("Creating the folder structure on Nextcloud ... ");

    // creating the missing folders on nextcloud and uploading the files in 4 threads to nextcloud
    if let Ok(files) = trave_dir_local(Path::new(&local_path), &mut root, &client, &extractor, depth.to_string()) {
        println!("done");

        // calculate the totat upload size
        let mut total_size: u64 = 0;
        for file in &files {
            total_size += file.get_size()
        }

        // create a shared nextcloud_client and a counter to track the upload progress and update the progress bar accordingly 
        let shared_client = Arc::new(client);
        let shared_uploaded_size: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

        // split the original vec 'files' in 4 seperate vecs and pass each one of them to a seperate thread
        let mut splitted_files: Vec<Vec<File>> = vec![];

        let halfs = split_vec_to_vecs(files);
        let mut quarters: Vec<(Vec<File>, Vec<File>)> = vec![];
        quarters.push(split_vec_to_vecs(halfs.0));
        quarters.push(split_vec_to_vecs(halfs.1));

        for q in quarters {
            splitted_files.push(q.0);
            splitted_files.push(q.1);
        }

        // spawning the uploading threads
        let mut threads: Vec<JoinHandle<()>> = vec![];
        for v in splitted_files {
            let uploaded_size = Arc::clone(&shared_uploaded_size);
            let client_cl2 = Arc::clone(&shared_client);
            threads.push(std::thread::spawn(move || {
                upload_folder(v, client_cl2, total_size, uploaded_size);
            }));
        }

        // joining the uploading threads
        for thread in threads {
            thread.join().expect("A thread could not be joined!");
        }

    } else {
        panic!("Error files")
    }
}