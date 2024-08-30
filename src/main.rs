mod nextcloud;
mod media;
mod filesystem;
mod helpers;
mod upload;

use nextcloud::NextcloudClient;
use media::Extractor;
use upload::sorted::upload_sorted;
use upload::unsorted::upload_unsorted;

use std::env;
use std::path::{Path, PathBuf};
use flexi_logger::{Logger, Duplicate, FileSpec, WriteMode};
use log::error;
use clap::{Arg, Command};
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
    let version = "0.1.0";

    // get parent folder of executable
    let mut exe_path: PathBuf;
    match env::current_exe() {
        Ok(val) => exe_path = val,
        Err(e) => {
            error!("{}", e);
            return
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

    // parser for cli options
    let matches = Command::new("nextsyncengine")
        .version(version)
        .about("Have a look at the README.md at https://github.com/SealJonny/nextsyncengine")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .propagate_version(true)
        .subcommand(
    Command::new("upload:sorted")
                .about("Uploads files from the specified folder to Nextcloud, organizing them into a structured folder hierarchy")
                .arg(
                    Arg::new("local_path")
                        .short('l')
                        .long("local_path")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("The path to the local folder that you want to upload"),
                )
                .arg(
                    Arg::new("remote_path")
                        .short('r')
                        .long("remote_path")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("The path on your Nextcloud server where the folder will be uploaded"),
                )
                .arg(
                    Arg::new("depth")
                        .short('d')
                        .long("depth")
                        .value_parser(clap::value_parser!(String))
                        .default_value("month")
                        .help("Sets the depth of the folder structure. Options are: year, month or day"),
                )
        )
        .subcommand(
    Command::new("upload:unsorted")
                .about("Uploads a local folder to Nextcloud recursively, placing all files directly in the specified root folder without preserving the local folder structure")
                .arg(
                    Arg::new("local_path")
                        .short('l')
                        .long("local_path")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("The path to the local folder that you want to upload"),
                )
                .arg(
                    Arg::new("remote_path")
                        .short('r')
                        .long("remote_path")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("The path on your Nextcloud server where the folder will be uploaded"),
                )
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
    // ToDo: add functionality to test if the credentials are valid


    match matches.subcommand() {
        Some(("upload:sorted", upload_matches)) => {
            // extract the options for upload:sorted
            let local_path = upload_matches.get_one::<String>("local_path").expect("required").trim().to_string();
            let remote_path = upload_matches.get_one::<String>("remote_path").expect("required").trim().to_string();
            
            let default_depth = &"month".to_string();
            let depth = upload_matches.get_one::<String>("depth").unwrap_or(default_depth).trim().to_string();
            
            // start the sorted upload of the folder at 'local_path' to 'remote_path'
            match upload_sorted(local_path, remote_path, depth, client, extractor) {
                Err(e) => error!("{}", e),
                _ => {}
            }
        }

        Some(("upload:unsorted", upload_matches)) => {
            // extract the options for upload:unsorted
            let local_path = upload_matches.get_one::<String>("local_path").expect("required").trim().to_string();
            let remote_path = upload_matches.get_one::<String>("remote_path").expect("required").trim().to_string();

            match upload_unsorted(local_path, remote_path, client, extractor) {
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