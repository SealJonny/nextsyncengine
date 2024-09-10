use reqwest::blocking::Client;
use std::time::Duration;
use std::fs::File as StdFile;
use std::vec;
use std::io::Read;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use xml::reader::{EventReader, XmlEvent};
use log::error;

use crate::filesystem::File;
use crate::helpers;

#[derive(Clone)]
pub struct NextcloudClient {
    url_server: String,
    url_dav: String,
    username: String,
    password: String,
    client: Client
}

impl NextcloudClient {

    pub fn new(url_server: String, username: String, password: String) -> NextcloudClient {
        let mut url_dav = url_server.clone();
        url_dav.push_str(format!("/remote.php/dav/files/{}", username).as_str());
        
        return NextcloudClient{
            url_server: url_server,
            url_dav: url_dav,
            username: username,
            password: password,
            client: Client::builder()
                .timeout(Duration::from_secs(2700))
                .build()
                .unwrap()
        }
    }

    // checks if nextcloud server is online. Returns an error if something went wrong on the client side
    pub fn is_online(&self) -> Result<bool, Box<dyn Error>> {
        let response = self.client.get(&self.url_server).send()?;

        // check status code for signes that the server is unavailable
        if response.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE || response.status() == reqwest::StatusCode::INTERNAL_SERVER_ERROR {
            return Ok(false)
        }
        Ok(true)
    }

    pub fn authenticate(&self) -> Result<bool, Box<dyn Error>> {
        let response = 
            self.client.get(&self.url_dav)
                .basic_auth(&self.username, Some(&self.password))
                .send()?;

        if response.status().is_success() {
            return Ok(true)
        }
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Ok(false)
        }
        if let Err(e) = response.error_for_status() {
            return Err(Box::new(e))
        }
        Ok(false)
    }
    
    // uploads a file to the specified location on a nextcloud server
    pub fn upload_file(&self, file: &File) -> Result<(), Box<dyn Error>> {
        // parse the file content into a vector needed to send the content via http request
        let local_path = file.get_local_path();
        let mtime = file.get_mtime();
        let remote_parent = file.get_remote_parent();

        let file_content = Self::read_file_to_vec(local_path)?;

        // extract the file name {from local_path and build the final url
        let url: String;
        if let Some(file_name) = local_path.file_name().and_then(|name| name.to_str()) {
            let remote_parent = helpers::path_to_str(remote_parent)?;
            url = self.build_url(vec![remote_parent.as_str(), file_name])

        } else {
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Extracting the file name from local path failed!")));
        }

        // send file to server using a http PUT request. The header 'X-OC-MTime' specifies the modification date which will be shown on the nextcloud UI
        let response = self.client.put(url.as_str())
            .header("X-OC-MTime", format!("{}", mtime))
            .basic_auth(&self.username, Some(&self.password))
            .body(file_content)
            .send()?;

        // checking reponse for errors
        self.evaluate_response_for_error(&response)

    }
    
    // lists a folder
    pub fn ls(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        // Todo: Optimize the error handling!
        
        let prop = r#"<?xml version='1.0'?> 
                    <d:propfind xmlns:d="DAV:" >
                        <d:prop>
                            <d:displayname />
                            <d:resourcetype />
                        </d:prop>
                    </d:propfind>
        "#;

        let url: String;
        let path = helpers::path_to_str(path)?;
        url = self.build_url(vec![path.as_str()]);

        let propfind = reqwest::Method::from_str("PROPFIND")?;

        let response = self.client.request(propfind, url)
            .header("Content-Type", "application/xml")
            .header("Depth", "1")
            .basic_auth(&self.username, Some(&self.password))
            .body(prop)
            .send()?;

        // checking the status code for erros
        if let Err(e) = self.evaluate_response_for_error(&response) {
            return Err(e)
        }
        
        // remove the root folder from the result
        let response = response.text()?;
        let mut folders = self.extract_folder_xml(&response)?;
        folders.remove(0);
        Ok(folders)
    }

    // queries the nextcloud sever if a folder at 'path' exists and returns the result
    pub fn exists_folder(&self, path: &Path) -> Result<bool, Box<dyn Error>> {

        // build the final url appending path to url_server
        let url: String;
        let path = helpers::path_to_str(path)?;
        url = self.build_url(vec![path.as_str()]);

        // query the server if this folder exists and returnig the erros directly to the caller of this method
        let propfind = reqwest::Method::from_str("PROPFIND")?;
        let response = self.client.request(propfind, url)
            .header("Depth", "0")
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        // checking the status code to determine if the folder exists or not
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(false)
        } 
        if response.status() == reqwest::StatusCode::MULTI_STATUS {
            return Ok(true)
        }

        // checking the responses for errors, if none are found the programm will be terminated
        // because the request returned an unexpected status code
        if let Err(e) = self.evaluate_response_for_error(&response) {
            return Err(e)
        }
        error!("exists_folder returned a unhandled response code!");
        panic!()
    }

    // creates a folder on the nextcloud server at the location 'path'
    pub fn create_folder(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        // build url containing the dav url and the location of the new folder
        let url: String;
        let path = helpers::path_to_str(path)?;
        url = self.build_url(vec![path.as_str()]);
        
        // creating the http method
        let mkcol = reqwest::Method::from_str("MKCOL")?;

        // sending the http request to make the folder at its destination
        let response = self.client.request(mkcol, url)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        self.evaluate_response_for_error(&response)
    }

    // evaluates the given response and determines if it has a error
    fn evaluate_response_for_error(&self, response: &reqwest::blocking::Response) -> Result<(), Box<dyn Error>> {
        if let Err(e) = response.error_for_status_ref() {
            return Err(Box::new(e))
        }
        Ok(())
    }

    // builds the url from the attribute 'url_server' and the given extensions
    fn build_url(&self, extensions: Vec<&str>) -> String {
        let mut current_url = self.url_dav.clone();
        for ext in extensions {
            if ext.starts_with("/") {
                current_url.push_str(&ext);
            } else {
                current_url.push_str(format!("/{}", &ext).as_str());
            }
        }
        current_url

    }

    // extracts the folder from xml data
    fn extract_folder_xml(&self, xml_data: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let parser = EventReader::from_str(xml_data);
        let mut inside_displayname = false;
        let mut current_displayname: Option<String> = None;
        let mut folders: Vec<PathBuf> = Vec::new();

        // loop through the xml tags
        for e in parser {
            match e {
                // check if current iteration is a start of an element
                Ok(XmlEvent::StartElement { name, ..})   => {
                    // if the local_name is displayname set inside_displayname to true for the next
                    // iteration to exract the data inside this tag
                    // if the local_name is 'collection' the current item with the name 'displayname' is a folder
                    // and will be pushed into folders
                    if name.local_name == "displayname" {
                        inside_displayname = true;

                    } else if name.local_name == "collection" {
                        if current_displayname.is_some() {
                            folders.push(Path::new(current_displayname.unwrap().as_str()).to_path_buf());
                            current_displayname = None;
                        }
                    }
                }

                // check if the current iteration is the data between the start and end of an element
                Ok(XmlEvent::Characters(data)) => {
                    if inside_displayname && !data.is_empty() {
                        current_displayname = Some(data);
                    }
                }

                // check if the current iteration is the end of an element
                Ok(XmlEvent::EndElement { name, .. }) => {
                    if name.local_name == "displayname" {
                        inside_displayname = false;
                    } else if name.local_name == "collection" {
                        current_displayname = None
                    }
                }

                Err(e) => {
                    return Err(Box::new(e))
                }
                _ => {}
            }
        }
        Ok(folders)
    }


    // reads a file to a vector and returns the vector
    fn read_file_to_vec(local_path: &Path) -> Result<Vec<u8>, io::Error> {
        let mut file = StdFile::open(local_path)?;
        let mut file_contents = Vec::with_capacity(file.metadata()?.len() as usize);
        file.read_to_end(&mut file_contents)?;
        Ok(file_contents)
    }
}



// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_upload_file_success() {
        // Create a mock for the PUT request to simulate the Nextcloud server
        let mut mock = mockito::Server::new();
        let server_url = mock.url();
        mock
            .mock("PUT", "/remote.php/dav/files/testuser/remote_parent/test_file.txt")
            .with_status(201)
            .with_header("Content-Type", "application/xml")
            .create();
    
        // Create a temporary directory and file to simulate a file upload
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let mut temp_file = StdFile::create(&file_path).unwrap();
        writeln!(temp_file, "This is a test file.").unwrap();
    
        // Create a mock Nextcloud file struct
        let mut fs_file = File::new(&file_path, 123456789);
        fs_file.set_remote_parent(PathBuf::from("/remote_parent"));
    
        // Initialize the Nextcloud client
        let nextcloud_client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());
    
        // Attempt to upload the file
        let result = nextcloud_client.upload_file(&fs_file);
    
        // Assert that the upload was successful
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_upload_file_error() {
        // Create a mock for the PUT request to simulate the Nextcloud server
        let mut mock = mockito::Server::new();
        let server_url = mock.url();
        mock
            .mock("PUT", "/remote.php/dav/files/testuser/remote_parent/testfile.txt")
            .with_status(201)
            .with_header("Content-Type", "application/xml")
            .create();
    
        // Create a temporary directory and file to simulate a file upload
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let mut temp_file = StdFile::create(&file_path).unwrap();
        writeln!(temp_file, "This is a test file.").unwrap();
    
        // Create a mock Nextcloud file struct
        let mut fs_file = File::new(&file_path, 123456789);
        fs_file.set_remote_parent(PathBuf::from("/remote_parent"));
    
        // Initialize the Nextcloud client
        let nextcloud_client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());
    
        // Attempt to upload the file
        let result = nextcloud_client.upload_file(&fs_file);
    
        // Assert that the upload failed with a 404 error
        assert!(result.is_err());
    }
    
    #[test]
    fn test_is_online_true() {
        // Create a mock for the GET request to simulate the Nextcloud server
        let mut mock = mockito::Server::new();
        let server_url = mock.url();
        mock
            .mock("GET", "/")
            .with_status(200)
            .with_header("Content-Type", "application/xml")
            .create();
    
        let client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());
    
        if let Ok(is_online) = client.is_online() {
            assert!(is_online);
        } else {
            panic!()
        }
    
    }
    
    #[test]
    fn test_is_online_false() {
        // Create a mock for the GET request to simulate the Nextcloud server
        let mut mock = mockito::Server::new();
        let server_url = mock.url();
        mock
            .mock("GET", "/")
            .with_status(503)
            .with_header("Content-Type", "application/xml")
            .create();
    
        let client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());
    
        if let Ok(is_online) = client.is_online() {
            assert_eq!(false, is_online);
        } else {
            panic!()
        }
    }
    
    #[test]
    fn test_authenticate_authorized() {
        // Create a mock for the GET request to simulate the Nextcloud server
        let mut mock = mockito::Server::new();
        let server_url = mock.url();
        mock
            .mock("GET", "/remote.php/dav/files/testuser")
            .with_status(200)
            .with_header("Content-Type", "application/xml")
            .create();
    
        let client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());
    
        if let Ok(logged_in) = client.authenticate() {
            assert!(logged_in);
        } else {
            panic!()
        }
    }
    
    #[test]
    fn test_authenticate_unauthorized() {
        // Create a mock for the GET request to simulate the Nextcloud server
        let mut mock = mockito::Server::new();
        let server_url = mock.url();
        mock
            .mock("GET", "/remote.php/dav/files/testuser")
            .with_status(401)
            .with_header("Content-Type", "application/xml")
            .create();
    
        let client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());
    
        if let Ok(logged_in) = client.authenticate() {
            assert_eq!(false, logged_in);
        } else {
            panic!()
        }
    }
}