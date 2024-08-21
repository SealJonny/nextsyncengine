use reqwest::blocking::Client;
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::io;
use std::path::Path;

pub struct NextcloudClient {
    url_server: String,
    username: String,
    password: String,
    client: Client
}

impl NextcloudClient {

    pub fn new(url_server: &str, username: &str, password: &str) -> NextcloudClient {
        return NextcloudClient{
            url_server: format!("{}/{}/{}", url_server, "remote.php/dav/files", username),
            username: username.to_string(),
            password: password.to_string(),
            client: Client::new()
        }
    }

    pub fn upload_file(&self, local_path: &Path, mtime: &i64) -> Result<(), Box<dyn Error>> {
        let file_name = local_path.file_name().and_then(|name| name.to_str());
        let file_content = Self::read_file_to_vec(local_path)?;
        // let mut file = File::open(local_path)?;
        // let mut file_contents = Vec::new();
        // file.read_to_end(&mut file_contents)?;

        let url: String;
        if file_name.is_some() {
            url = format!("{}/{}", self.url_server, file_name.unwrap());
        } else {
            println!("Path is not valid");
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Path could not be converted to &str.")));
        }
        let response = self.client.put(url.as_str())
            .header("X-OC-MTime", format!("{}", mtime))
            .basic_auth(&self.username, Some(&self.password))
            .body(file_content)
            .send()?;

        if response.status().is_success() {
            println!("It worked!");
            Ok(())
        } else {
            println!("Test: {}", response.status());
            Err(Box::new(io::Error::new(io::ErrorKind::Other, "Upload failed")))
        }
    }

    fn read_file_to_vec(local_path: &Path) -> Result<Vec<u8>, io::Error> {
        let mut file = File::open(local_path)?;
        let mut file_contents = Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut file_contents)?;
        Ok(file_contents)
    }
}