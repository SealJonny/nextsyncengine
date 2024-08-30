use std::error::Error;
use std::io;
use std::io::Write;


pub fn progress_bar(iteration: u64, total: u64, prefix: &str, suffix: &str) {
    let fill = '█';
    let length = 50;
    let percent = 100.0 * (iteration as f64 / total as f64);
    let filled_length = (length as f64 * iteration as f64 / total as f64).round() as usize;
    let bar = fill.to_string().repeat(filled_length) + &"-".repeat(length - filled_length);

    print!("\r{} |{}| {:.1}% {}", prefix, bar, percent, suffix);
    io::stdout().flush().unwrap();

    if iteration == total {
        println!();
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