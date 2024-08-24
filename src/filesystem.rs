use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::error::Error;

pub struct Folder{
    name: String,
    sub_folders: Vec<Folder>
}

impl Folder {
    pub fn new(name: String) -> Self {
        return Folder {
            name: name,
            sub_folders: Vec::new()
        }
    }

    // wrapper for recursive method add_sub_folder_intern
    pub fn add_sub_folder(&mut self, folder: Folder, path_parent: &Path) {
        // split the path into the single folders and push them into a vec
        let root = Path::new(&self.name);
        let mut iter_root = root.iter();
        let mut folders: Vec<String> = Vec::new();
        for f in path_parent.iter() {
            if let Some(root_str) = iter_root.next() {
                if f == root_str {
                    continue
                }
            }
            if let Some(val) = f.to_str() {
                folders.push(val.to_string());
            }
        }

        // reverse the order to later use .pop() to extract the path from the beginning and not from the end
        folders.reverse();
        // Todo: Remove root.name from folders
        self.add_sub_folder_intern(folder, &mut folders)
    }

    // recursive method which adds the given folder to the sub_folders of the specified parent folder
    pub fn add_sub_folder_intern(&mut self, folder: Folder, parent_folders: &mut Vec<String>) {
        // check if .pop() returns not None
        if let Some(sub_folder_name) = parent_folders.pop() {
            // fetch the folder with the name 'sub_folder_name'. If it returns a folder perform the same function on it
            if let Some(sub_folder) = self.get_subfolder(sub_folder_name) {
                sub_folder.add_sub_folder_intern(folder, parent_folders)
            }
        } else {
            // if .get_subfolder() returns None add the folder to the sub_folders of the current folder
            self.sub_folders.push(folder);
        }
        
    }

    // returns the subfolder of the current folder with the specified name
    fn get_subfolder(&mut self, name: String) -> Option<&mut Folder> {
        for folder in &mut self.sub_folders {
            if folder.name == name {
                return Some(folder)
            }
        }
        None
    }

    pub fn convert_to_string(&self, depth: i32) -> String {
        let mut result = String::new();
        let mut indentation = String::new();
        for _i in 0..depth - 1 {
            indentation.push_str("\t");
        }
        result.push_str(format!("{}- {}\n", indentation, self.name.clone()).as_str());

        for f in self.sub_folders.iter() {
            let next_result = f.convert_to_string(depth + 1);
            result.push_str(format!("\t{}", &next_result).as_str());
        }

        result
    }
}

pub struct File {
    local_path: PathBuf,
    remote_parent: PathBuf,
    mtime: i64
}

impl File {
    pub fn new(local_path: &Path, remote_parent: &Path) -> File {
        return File {
            local_path: local_path.to_owned(),
            remote_parent: remote_parent.to_owned(),
            mtime: 0
        }
    }

    // queries the filesystem to get the file size
    pub fn get_size(&self) -> Result<u64, Box<dyn Error>> {
        let meta_data = self.local_path.metadata()?;
        Ok(meta_data.size())
    }

    pub fn get_local_path(&self) -> &Path {
        &self.local_path
    }
    
    pub fn get_remote_parent(&self) -> &Path {
        &self.remote_parent
    }

    pub fn get_mtime(&self) -> i64 {
        self.mtime
    }

    // sets the attribute 'mtime' to the value of the parameter 'mtime'
    pub fn set_mtime(&mut self, mtime: i64) {
        self.mtime = mtime
    }


    
}