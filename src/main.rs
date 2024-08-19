mod nextcloud;

use nextcloud::NextcloudClient;
use std::env;
use std::path::Path;

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

    println!("{}:{}@{}", username, password, server_url);

    let client = NextcloudClient::new(&server_url, &username, &password);
    
    match client.upload_file("Vineyard.jpg") {
        Ok(_val) => print!("Successfully uploaded demo!"),
        Err(e) => eprint!("Error while uploading demo: {}", e)
    }
}
