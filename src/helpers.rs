use std::io;
use std::io::Write;
use std::error::Error;
use std::path::Path;


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

// convertes a &Path to &str
pub fn path_to_str(path: &Path) -> Result<String, Box<dyn Error>> {
    if let Some(path_str) = path.to_str() {
        Ok(path_str.to_string())
    } else {
        Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "path could not be converted to string!")))
    }
}