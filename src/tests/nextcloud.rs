#[cfg(test)]
mod tests {
    use std::fs::File as StdFile;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use crate::nextcloud::NextcloudClient;
    use crate::filesystem::File as FilesystemFile;

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
        let mut fs_file = FilesystemFile::new(&file_path, 123456789);
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
        let mut fs_file = FilesystemFile::new(&file_path, 123456789);
        fs_file.set_remote_parent(PathBuf::from("/remote_parent"));

        // Initialize the Nextcloud client
        let nextcloud_client = NextcloudClient::new(server_url, "testuser".to_string(), "password".to_string());

        // Attempt to upload the file
        let result = nextcloud_client.upload_file(&fs_file);

        // Assert that the upload failed with a 404 error
        assert!(result.is_err());
    }
}
