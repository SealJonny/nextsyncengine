use reqwest::blocking::Client;
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::path::Path;

pub struct NextcloudClient {
    url_server: String,
    client: Client
}

impl NextcloudClient {

    pub fn new(url_server: &str) -> NextcloudClient {
        return NextcloudClient{
            url_server: url_server.to_string(),
            client: Client::new()
        }
    }

    pub fn upload_file(&self, local_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(local_path);
        let mut file = File::open(path)?;
        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents)?;

        let response = self.client.put(self.url_server.as_str())
            .basic_auth("admin", Some("admin"))
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