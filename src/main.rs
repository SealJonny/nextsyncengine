mod nextcloud;
mod media;

use nextcloud::NextcloudClient;
use std::env;
use std::path::Path;
use media::Extractor;

fn main() {
    let path = Path::new(".env");
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

    //println!("{}:{}@{}", username, password, server_url);

    let client = NextcloudClient::new(&server_url, &username, &password);

    let path = Path::new("/home/sealjonny/Github/nextsyncengine/Vineyard.jpg");
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        println!("file: {}", file_name);
    } else {
        println!("file: <default>");
    }
    
    let image = Path::new("/home/sealjonny/Github/nextsyncengine/Vineyard.jpg");

    let ext = Extractor::new("/usr/local/bin/exiftool-amd64-glibc");

    let mut mtime: i64 = 0;
    if let Err(e) = ext.extract_date_time(image).map(|val| mtime = val) {
        eprintln!("{}", e);
        return
    }

    match client.upload_file(image, &mtime) {
        Ok(_val) => print!("Successfully uploaded demo!"),
        Err(e) => eprint!("Error while uploading demo: {}", e)
    }
}
