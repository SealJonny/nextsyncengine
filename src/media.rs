use std::process::Command;
use std::path::Path;
use std::{io, vec};
use chrono::NaiveDateTime;
use std::error::Error;

pub struct Extractor {
    exiftool: String,
    supported_formats: Vec<String>
}

impl Extractor {

    pub fn new(exiftool: String) -> Self {
        Self {
            exiftool: exiftool,
            supported_formats: vec![]
        }
    }

    pub fn get_supported_formats(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        let cmd = format!("{} -listwf | sed '1d'", &self.exiftool);

        #[cfg(windows)]
        let cmd = format!("{} -listwf | Select-Object -Skip 1", &self.exiftool);

        let result = self.execute_shell_command(cmd)?;
        let result = result.replace(" ", "\n");
        let mut result: Vec<String> = result
                .split_ascii_whitespace()
                .map(|val| val.to_lowercase())
                .collect();
        self.supported_formats.append(&mut result);
        Ok(())
    }

    // returns true if the file is supported by exiftool
    fn is_supported_by_exif(&self, path: &Path) -> bool {
        // checking if the extension of the file is contained in IMAGE_FORMATS or VIDEO_FORMATS
        if let Some(ext) = path.extension().and_then(|val| val.to_str()) {
            return self.supported_formats.contains(&ext.to_lowercase());
        }
        false
    }

    // returns the modification date of a file as an unix timestamp. Returns an error if path does not point to a file
    pub fn extract_date_time(&self, path: &Path) -> Result<i64, Box<dyn std::error::Error>> {
        // checking if path points to a file and if not returnig an error
        if !path.is_file() {
            return Err(Box::new(
                io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Path: {} is not a file!", path.display()))));
        }

        // checking if file is supported by exiftool if not using os to get mtime
        if self.is_supported_by_exif(path) {
            return self.extract_date_time_exif(path)
        } else {
            return self.extract_date_time_os(path)
        }
    }
    
    // extracts the modification date using the os
    fn extract_date_time_os(&self, path: &Path) -> Result<i64, Box<dyn std::error::Error>> {
        // handling potential error and returning mtime if present
        match get_metadata(path.to_str().unwrap()) {
            Ok(meta_data) => Ok(meta_data.get_mtime()),
            Err(e) => Err(e)
        }

    }

    // extracts the modification date using the exiftool binary
    fn extract_date_time_exif(&self, path: &Path) -> Result<i64, Box<dyn std::error::Error>> {
        // converting the extracted date time string into a unix timestamp
        let path_str: String;
        if let Some(tmp) = path.to_str() {
            path_str = tmp.to_string();
        } else {
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, "Failed to extract exif metadata due to a conversion error")))
        }

        #[cfg(unix)]
        let cmd = format!("{} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTime -ModifyDate -FileModifyDate \"{}\"", &self.exiftool, path_str);

        #[cfg(windows)]
        let cmd = format!("{} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTime -ModifyDate -FileModifyDate \"{}\"", &self.exiftool, path_str);

        // extract the date time from the file using exiftool
        let result = self.execute_shell_command(cmd)?;
        let result = result.replace("\r\n", "\n");

        // only use the first found time by exiftool
        let mut times = result
        .split("\n") 
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
        let mut result = times.swap_remove(0);

        // use the next extracted date time if the current date time is useless
        while result == "0000:00:00 00:00:00" && !times.is_empty() {
            result = times.swap_remove(0);
        }

        // attempt to convert result to a datetime object
        let format = "%Y:%m:%d %H:%M:%S";
        match NaiveDateTime::parse_from_str(&result, format) {
            Ok(mtime) => Ok(mtime.and_utc().timestamp()),
            Err(_e) => Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("Failed to convert {} to a date time, {}", result, path_str))))
        }
    }

    // execute the given cmd in the systems shell and return stdout
    fn execute_shell_command(&self, cmd: String) -> Result<String, Box<dyn std::error::Error>> {
        // execute 'cmd' in the systems shell
        #[cfg(unix)]
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect(format!("Failed to execute command {}", cmd).as_str());
            
        #[cfg(windows)]
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(&cmd)
            .output()
            .expect(format!("Failed to execute command {}", cmd).as_str());

        // convert the ouput to a string and remove trailing whitespaces, line breackers or indents
        let stdout = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim_end().to_string();

        // return stdout if shell terminated successfully, otherwise return stderr
        if !stdout.is_empty() && output.status.success() {
            Ok(stdout)
        } else {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to extract metadata with exiftool: Exit Code {}, CMD: {}, {}", output.status.code().unwrap(), stderr, cmd)
                    )
                )
            )
        }
    }

}

// custom Metadata struct for storing the mtime and size of a file
pub struct CustomMetadata {
    mtime: i64,
    size: u64
}

impl CustomMetadata {
    pub fn new(mtime: i64, size: u64) -> CustomMetadata {
        return CustomMetadata {
            mtime: mtime,
            size: size
        }
    }

    pub fn get_mtime(&self) -> i64 {
        return self.mtime
    }

    pub fn get_size(&self) -> u64 {
        return self.size
    }
}

// unix specific function to extract metadata from a file
#[cfg(unix)]
pub fn get_metadata(path: &str) -> Result<CustomMetadata, Box<dyn Error>> {
    use std::path::Path;
    use std::os::unix::fs::MetadataExt;

    let path = Path::new(path);

    let metadata = path.metadata()?;

    Ok(CustomMetadata::new(metadata.mtime(), metadata.size()))
}

// windows specific function to extract metadata from a file
#[cfg(windows)]
pub fn get_metadata(path: &str) -> Result<CustomMetadata, Box<dyn Error>> {
    use std::fs::metadata;
    let metadata = metadata(path)?;
    let size: u64 = metadata.len().try_into().unwrap();

    // extract modified date and convert it to a unix timestamp
    let mod_time = metadata.modified()?;
    let duration_since_epoche = mod_time.duration_since(std::time::UNIX_EPOCH)?;
    let mtime: i64 = duration_since_epoche.as_secs().try_into().unwrap();

    Ok(CustomMetadata::new(mtime, size))
}

// #[cfg(test)]
//  mod tests {
//      use core::panic;

//      use super::*;

//      #[test]
//      fn test_get_supported_formats() {
//          let mut extractor = Extractor::new("c:\\nextsyncengine\\exiftool.exe".to_string());
//          match extractor.get_supported_formats() {
//              Ok(_val) => {
//                  println!("{:?}", extractor.supported_formats);
//                  assert!(true)
//             }
//              Err(e) => panic!("{}", e)
//          }
//      }

//      #[test]
//      fn test_is_supported_exif() {
//          let mut extractor = Extractor::new("c:\\nextsyncengine\\exiftool.exe".to_string());
//          let _ = extractor.get_supported_formats();
//          assert!(extractor.is_supported_by_exif(Path::new("hallo.jpg")))

//      }
// }