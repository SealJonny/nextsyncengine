use reqwest::blocking::Client;
use std::fmt::format;
use std::fs::File;
use std::io::Read;
use std::error::Error;

pub struct NextcloudClient {
    url_server: String,
    username: String,
    password: String,
    client: Client
}

impl NextcloudClient {

    pub fn new(url_server: &str, username: &str, password: &str) -> NextcloudClient {
        return NextcloudClient{
            url_server: url_server.to_string() + "/remote.php/dav/files/" + username,
            username: username.to_string(),
            password: password.to_string(),
            client: Client::new()
        }
    }

    pub fn upload_file(&self, local_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(local_path)?;
        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents)?;

        let url = format!("{}/{}", self.url_server, local_path);
        let response = self.client.put(url.as_str())
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .body(file_contents)
            .send()?;

        if response.status().is_success() {
            print!("It worked!");
            Ok(())
        } else {
            print!("Test: {}", response.status());
            Err(Box::new(response.error_for_status().unwrap_err()))
        }
    }
}