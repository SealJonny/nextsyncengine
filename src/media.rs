use std::process::Command;
use std::path::Path;
use std::io;
use chrono::NaiveDateTime;

pub struct Extractor {
    exiftool: String
}

impl Extractor {

    pub fn new(exiftool: &str) -> Extractor {
        Extractor {
            exiftool: exiftool.to_string()
        }
    }

    pub fn extract_date_time(&self, path: &Path) -> Result<i64, Box<dyn std::error::Error>> {
        if !path.is_file() {
            return Err(Box::new(
                io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Path: {} is not a file!", path.display()))));
        }
        
        let result: String;
        if let Some(path_str) = path.to_str() {
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!("{} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTimeOriginal -DateCreated -CreateDate -FileCreateDate '{}' | head -n 1", &self.exiftool, path_str))
                .output()
                .expect("Failed to execute command");
            
            // Convert the output to a String and print it
            let stdout = String::from_utf8_lossy(&output.stdout);
            result = stdout.trim_end().to_string();
        } else {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to convert path to &str!")))
        }


        let format = "%Y:%m:%d %H:%M:%S";
        match NaiveDateTime::parse_from_str(&result, format) {
            Ok(val) => Ok(val.and_utc().timestamp()),
            Err(e) => Err(Box::new(e))
        }
    }
}
