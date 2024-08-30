mod nextcloud;
mod media;
mod filesystem;
mod helpers;
mod upload;

use nextcloud::NextcloudClient;
use media::Extractor;
use upload::sorted::upload_sorted;

use std::env;
use std::path::Path;
use flexi_logger::{Logger, Duplicate, FileSpec, WriteMode};
use log::error;
use clap::{Arg, Command};


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

    // parser for cli options
    let matches = Command::new("nextsyncengine")
        .version(version)
        .about("Have a look at the README.md at https://github.com/SealJonny/nextsyncengine")
        .subcommand(
    Command::new("upload:sorted")
                .about("Uploads the files of specified folder to a organized structure on Nextcloud")
                .arg(
                    Arg::new("local_path")
                        .short('l')
                        .long("local_path")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("The path to your local folder which will be uploaded"),
                )
                .arg(
                    Arg::new("remote_path")
                        .short('r')
                        .long("remote_path")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("The location where on your Nextcloud server the folder will be uploaded"),
                )
                .arg(
                    Arg::new("depth")
                        .short('d')
                        .long("depth")
                        .value_parser(clap::value_parser!(String))
                        .default_value("month")
                        .help("Options are: year, month, and day. Determines the depth of the folder structure."),
                )
            )
        .get_matches();
    
    
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
        _ => {
            error!("The command line options could not be parsed!");
            return
        }
    }
}