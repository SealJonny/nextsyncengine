use std::process::Command;
use std::path::Path;
use std::io;
use chrono::NaiveDateTime;
use std::error::Error;

const IMAGE_FORMATS: [&str; 43] = [
    ".jpg", ".jpeg", ".tif", ".tiff", ".gif", ".bmp", ".png", ".ppm",
    ".pgm", ".pbm", ".pnm", ".webp", ".heif", ".heic", ".jp2", ".j2k",
    ".jpf", ".jpx", ".jpm", ".mj2", ".ico", ".cr2", ".cr3", ".nef",
    ".nrw", ".orf", ".raf", ".arw", ".rw2", ".dng", ".sr2", ".3fr",
    ".rwl", ".mrw", ".raw", ".pef", ".iiq", ".k25", ".kc2", ".erf",
    ".srw", ".x3f", ".svg"
];

const VIDEO_FORMATS: [&str; 36] = [
    ".mp4", ".mov", ".avi", ".mkv", ".3gp", ".3g2", ".wmv", ".asf",
    ".flv", ".f4v", ".swf", ".m2ts", ".mts", ".m2t", ".ts", ".mxf",
    ".mpg", ".mpeg", ".mpe", ".mpv", ".m4v", ".m4p", ".rm", ".rmvb",
    ".webm", ".ogv", ".ogg", ".ogx", ".dv", ".dif", ".m2v", ".qt",
    ".mjpg", ".mj2", ".gif", ".mov"
];

pub struct Extractor<'a> {
    exiftool: String,
    supported_images: &'a [&'a str; 43],
    supported_videos: &'a [&'a str; 36]
}

impl<'a> Extractor<'a> {

    pub fn new(exiftool: String) -> Self {
        Self {
            exiftool: exiftool,
            supported_images: &IMAGE_FORMATS,
            supported_videos: &VIDEO_FORMATS
        }
    }

    // returns true if the file is supported by exiftool
    fn is_supported_by_exif(&self, path: &Path) -> bool {
        // checking if the extension of the file is contained in IMAGE_FORMATS or VIDEO_FORMATS
        if let Some(ext) = path.extension().and_then(|val| val.to_str()) {
            let ext_full = format!(".{}", ext.to_lowercase());
            return self.supported_images.contains(&ext_full.as_str()) ||
                   self.supported_videos.contains(&ext_full.as_str());
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
        // convert path to str and execute exiftool bash command to extract the mtime from the file
        let result: (String, String);
        
        if let Some(path_str) = path.to_str() {
            #[cfg(unix)]
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!("{} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTime -ModifyDate -FileModifyDate '{}' | head -n 1", &self.exiftool, path_str))
                .output()
                .expect("Failed to execute command");
            
            #[cfg(windows)]
            let output = Command::new("powershell")
                .arg("-Command")
                .arg(format!("{} -m -s3 -d '%Y:%m:%d %H:%M:%S' -DateTime -ModifyDate -FileModifyDate '{}' | Select-Object -First 1", &self.exiftool, path_str))
                .output()
                .expect("Failed to execute command");

            // convert the ouput to a string and remove trailing whitespaces, line breackers or indents
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            result = (stdout.trim_end().to_string(), stderr.trim_end().to_string());
        } else {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to convert path to &str!")))
        }
        // converting the extracted date time string into a unix timestamp
        let format = "%Y:%m:%d %H:%M:%S";
        match NaiveDateTime::parse_from_str(&result.0, format) {
            Ok(mtime) => Ok(mtime.and_utc().timestamp()),
            Err(_e) => Err(Box::new(io::Error::new(io::ErrorKind::Other, result.1)))
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