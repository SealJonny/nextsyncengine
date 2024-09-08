mod nextcloud;
mod media;
mod filesystem;
mod helpers;
mod upload;

use clap::builder::ValueParser;
use nextcloud::NextcloudClient;
use media::Extractor;
use upload::sorted::upload_sorted;
use upload::unsorted::upload_unsorted;

use std::env;
use std::path::{Path, PathBuf};
use flexi_logger::{Logger, Duplicate, FileSpec, WriteMode};
use log::error;
use clap::{Arg, ArgGroup, Command};
use colored::*;


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

fn main() {
    // get the current working directory
    let working_dir: PathBuf;
    match env::current_dir() {
        Ok(val) => working_dir = val,
        Err(e) => {
            error!("Could not determine the current working directory: {}", e);
            panic!()
        }
    }
    
    // get parent folder of executable
    let mut exe_path: PathBuf;
    match env::current_exe() {
        Ok(val) => exe_path = val,
        Err(e) => {
            error!("{}", e);
            panic!()
        }
    }
    exe_path.pop();

    let config_folder = exe_path.join("_nextsyncengine_");
    init_logger(&config_folder);
    let path = config_folder.join(".env");
    dotenv::from_path(path).expect("Failed to read .env file");

    // Helper function to retrieve environment variables
    fn get_env_var(var_name: &str) -> String {
        env::var(var_name).unwrap_or_else(|e| {
            error!("Error while reading '{}': {}", var_name, e);
            String::new()
        })
    }

    let server_url = get_env_var("SERVER_URL");
    let username = get_env_var("NC_USERNAME");
    let password = get_env_var("PASSWORD");
    let exiftool = get_env_var("EXIFTOOL");

    let client = NextcloudClient::new(server_url, username.clone(), password);
    let extractor = Extractor::new(exiftool);

    // common args between upload:sorted and upload:unsorted
    let local_arg =
        Arg::new("local")
            .short('l')
            .long("local")
            .value_parser(clap::value_parser!(String))
            .help("Path to a local folder containing the files you want to upload.");

    let remote_arg = 
        Arg::new("remote")
            .short('r')
            .long("remote")
            .value_parser(clap::value_parser!(String))
            .required(true)
            .help("Path to the location on Nextcloud where your files will be uploaded too.");

    let threads_arg =
        Arg::new("threads")
            .short('t')
            .long("threads")
            .value_parser(ValueParser::new(|s: &str| {
                let value: usize = s.parse().map_err(|_| format!("{} isn't a valid number", s))?;
                if value < 1 || value > 6{
                    return Err(format!("The number of threads must be between 1 and 6, but '{}' was provided", value))
                }
                Ok(value)
            }))
            .default_value("3")
            .help("Lets you control the number of threads used to upload the files. The value must be between 1 and 6.");
    
    let file_arg = 
        Arg::new("file")
            .short('f')
            .long("file")
            .value_parser(ValueParser::new(|s: &str| {
                let value: String = s.parse().map_err(|_| format!("{} isn't a valid String", s))?;
                let path = Path::new(&value);
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "txt" {
                            return Ok(value)
                        }
                        return Err(format!("The provided path {} does not point to a .txt file", value))
                    }
                    return Err(format!("Failed to extract the file extension from  {}", value))
                }
                Err(format!("The provided path {} does not point to a file", value))
            }))
            .help("Path to the text file generated by nextsyncengine or any other text file with the same format.");

    let local_required_group = 
        ArgGroup::new("local_required_group")
            .args(&["local", "file"])
            .required(true)
            .multiple(false);

    // parser for cli options
    let matches = Command::new("nextsyncengine")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Have a look at the README.md at https://github.com/SealJonny/nextsyncengine")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .propagate_version(true)
        .subcommand(
    Command::new("upload:sorted")
                .about("Allows you to upload files from a local folder and its sub folders to a folder structure organized by date on Nextcloud.")
                .arg(local_arg.clone())
                .arg(file_arg.clone())
                .arg(remote_arg.clone())
                .arg(
                    Arg::new("depth")
                        .short('d')
                        .long("depth")
                        .value_parser(clap::value_parser!(String))
                        .default_value("month")
                        .help("Lets you control the depth of the remote folder structure. Options are: year, month and day."),
                )
                .arg(threads_arg.clone())
                .group(local_required_group.clone())
        )
        .subcommand(
    Command::new("upload:unsorted")
                .about("Allows you to upload files from a local folder and its sub folders to Nextcloud while getting rid of the original folder structure.")
                .arg(local_arg.clone())
                .arg(file_arg.clone())
                .arg(remote_arg.clone())
                .arg(threads_arg.clone())
                .group(local_required_group.clone())
        )
        .get_matches();
    
    
    // checking if nextcloud server is online and not in maintenance mode and terminating execution if it is offline.
    print!("{}", "Checking if Nextcloud server is online ... ".green());
    match client.is_online() {
        Ok(val) => {
            if !val {
                println!("{}", "\nNextcloud server is offline or in maintenance mode!".red());
                return
            }
            println!("{}", "done".green())
        }
        Err(e) => {
            error!("{}", e);
            return
        }
    }
    
    // check if the credentials in the .env are valid
    match client.authenticate() {
        Ok(true) => println!("{}", format!("You are logged in as {}.", &username).green()),
        Ok(false) => {
            println!("{}", "Your Nextcloud credentials are wrong. Check your .env!".red());
            return
        }
        Err(e) =>  {
            error!("{}", e);
            return
        }
    }

    // check which command was used by the user
    match matches.subcommand() {
        Some(("upload:sorted", upload_matches)) => {
            // extract the options for upload:sorted
            let local_path = upload_matches.get_one::<String>("local");
            let file_path = upload_matches.get_one::<String>("file");
            let remote_path = upload_matches.get_one::<String>("remote").expect("--remote is required").trim().to_string();
            let depth = upload_matches.get_one::<String>("depth").expect("--depth was not set").trim().to_string();
            let num_threads = upload_matches.get_one::<usize>("threads").expect("--threads was not set");

            // determine if user chose local arg or file arg
            let mut path_upload = String::new();
            let from_folder = helpers::get_path_folder_or_file(&mut path_upload, local_path, file_path, &working_dir);

            // start the sorted upload of the files from 'path_upload' to 'remote_path'
            match upload_sorted(path_upload, from_folder, remote_path, depth, *num_threads, client, extractor) {
                Err(e) => error!("{}", e),
                _ => {}
            }
        }

        Some(("upload:unsorted", upload_matches)) => {
            // extract the options for upload:unsorted
            let local_path = upload_matches.get_one::<String>("local");
            let file_path = upload_matches.get_one::<String>("file");
            let remote_path = upload_matches.get_one::<String>("remote").expect("--remote is required").trim().to_string();
            let num_threads = upload_matches.get_one::<usize>("threads").expect("--threads was not set");

            // determine if user chose local arg or file arg
            let mut path_upload = String::new();
            let from_folder = helpers::get_path_folder_or_file(&mut path_upload, local_path, file_path, &working_dir);
            
            // start the unsorted upload of the files from 'path_upload' to 'remote_path'
            match upload_unsorted(path_upload, from_folder, remote_path, *num_threads, client, extractor) {
                Err(e) => error!("{}", e),
                _ => {}
            }
        }
        _ => {
            error!("The command line options could not be parsed!");
            return
        }
    }
}