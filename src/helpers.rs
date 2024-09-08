use std::io;
use std::io::Write;
use std::error::Error;
use std::path::Path;
use log::error;


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

// If the path starts with \\?\, remove it
fn remove_extended_prefix(path: String) -> String {
    if path.starts_with(r"\\?\") {
        path.trim_start_matches(r"\\?\").to_string()
    } else {
        path
    }
}

// determines the path to the folder containing the files or a file containing the paths to the files which will be uploaded
// returning whether the path points to the folder a the file
pub fn get_path_folder_or_file(path_upload: &mut String, local_path: Option<&String>, file_path: Option<&String>, working_dir: &Path) -> bool {
    let mut from_folder = true;
    if let Some(local) = local_path {
        // resolving local to a absolute path
        let combinded_path = working_dir.join(&local);
        let resolved_path = match combinded_path.canonicalize() {
            Ok(absolute_path) => {
                let absolute_path = remove_extended_prefix(absolute_path.to_str().unwrap().to_string());
                absolute_path
            }
            Err(e) => {
                error!("Failed resolving {} to an absolute path", e);
                String::new()
            }
        };
        // terminate programm if local could not be resolved to a absolute path
        if resolved_path.is_empty() {
            panic!()
        }
        *path_upload = resolved_path;
    } else if let Some(file) = file_path {
        from_folder = false;
        *path_upload = file.to_string()
    } else {
        error!("--local or --file is requried");
        panic!()
    };

    from_folder
}