use std::path::{Path, PathBuf};

use crate::media::get_metadata;

#[derive(Debug)]
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
        // split the path into the single folders, remove the root folder and push them into a vec
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
            if let Some(sub_folder) = self.get_subfolder_mut(sub_folder_name) {
                sub_folder.add_sub_folder_intern(folder, parent_folders)
            }
        } else {
            // if .get_subfolder() returns None add the folder to the sub_folders of the current folder
            self.sub_folders.push(folder);
        }
        
    }

    // wrapper for recursive method has_subfolder_intern
    pub fn has_subfolder(&self, path_folder: &Path) -> bool {
        // split the path into the single folders, remove the root folder and push them into a vec
        let root = Path::new(&self.name);
        let mut iter_root = root.iter();
        let mut folders: Vec<String> = Vec::new();
        for f in path_folder.iter() {
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
        return self.has_subfolder_intern(&mut folders)
    }

    // recursively returns whether a folder and its sub folders have a sub folder 'path_folder'
    fn has_subfolder_intern(&self, folders: &mut Vec<String>) -> bool {
        // if folders is not empty and .get_subfolder() returns a Folder the sub folder 'path_folder' exists
        if let Some(sub_folder_name) = folders.pop() {
            if let Some(sub_folder) = self.get_subfolder(sub_folder_name) {
                if folders.len() == 0 {
                    return true
                }
                // search the next subfolder
                return sub_folder.has_subfolder_intern(folders)

            } else {
                return false
            }
        }
        false
        
    }
    
    fn get_subfolder(&self, name: String) -> Option<&Folder> {
        for folder in &self.sub_folders {
            if folder.name == name {
                return Some(folder)
            }
        }
        None
    }
    // returns the subfolder of the current folder with the specified name
    fn get_subfolder_mut(&mut self, name: String) -> Option<&mut Folder> {
        for folder in &mut self.sub_folders {
            if folder.name == name {
                return Some(folder)
            }
        }
        None
    }

    // pub fn convert_to_string(&self, depth: i32) -> String {
    //     let mut result = String::new();
    //     let mut indentation = String::new();
    //     for _i in 0..depth - 1 {
    //         indentation.push_str("\t");
    //     }
    //     result.push_str(format!("{}- {}\n", indentation, self.name.clone()).as_str());

    //     for f in self.sub_folders.iter() {
    //         let next_result = f.convert_to_string(depth + 1);
    //         result.push_str(format!("\t{}", &next_result).as_str());
    //     }

    //     result
    // }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl PartialEq for Folder {
    fn eq(&self, other: &Self) -> bool {
        // check for equality by comparing names and numbers of sub folders
        if self.sub_folders.len() != other.sub_folders.len() || self.name != other.name {
            return false
        }

        // compare each pair of sub folders to ensure equality for the complete structure
        for (index, sub_self) in self.sub_folders.iter().enumerate() {
            if let Some(sub_other) = other.sub_folders.get(index) {
                if sub_self.eq(sub_other) == false {
                    return false
                }
            }
        }
        true
    }
}

#[derive(Clone)]
pub struct File {
    local_path: PathBuf,
    remote_parent: PathBuf,
    mtime: i64,
    size: u64
}

impl File {
    pub fn new(local_path: &Path, mtime: i64) -> File {
        let mut size: u64 = 0;
        if let Ok(meta_data) = get_metadata(local_path.to_str().unwrap()) {
            size = meta_data.get_size();
        }
        return File {
            local_path: local_path.to_owned(),
            remote_parent: PathBuf::new(),
            mtime: mtime,
            size: size
        }
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_local_path(&self) -> &Path {
        &self.local_path
    }
    
    pub fn get_remote_parent(&self) -> &Path {
        &self.remote_parent
    }

    pub fn set_remote_parent(&mut self, remote_parent: PathBuf) {
        self.remote_parent.push(remote_parent);
    }

    pub fn get_mtime(&self) -> i64 {
        self.mtime
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_sub_folder() {
        // create a object of the Folder struct manually
        let mut root_manual = Folder::new("root".to_string());
        root_manual.sub_folders.push(Folder::new("sub_1".to_string()));

        if let Some(sub_1) = root_manual.get_subfolder_mut("sub_1".to_string()) {
            sub_1.sub_folders.push(Folder::new("sub_sub_1".to_string()));
            sub_1.sub_folders.push(Folder::new("sub_sub_2".to_string()));

            if let Some(sub_sub_2) = sub_1.get_subfolder_mut("sub_sub_2".to_string()) {
                sub_sub_2.sub_folders.push(Folder::new("sub_sub_sub_1".to_string()));
            } 
        }

        // create a object of the Folder struct with the 'add_sub_folder' method
        let mut root_auto = Folder::new("root".to_string());
        root_auto.add_sub_folder(Folder::new("sub_1".to_string()), Path::new("root"));
        root_auto.add_sub_folder(Folder::new("sub_sub_1".to_string()), Path::new("root/sub_1"));
        root_auto.add_sub_folder(Folder::new("sub_sub_2".to_string()), Path::new("root/sub_1"));
        root_auto.add_sub_folder(Folder::new("sub_sub_sub_1".to_string()), Path::new("root/sub_1/sub_sub_2"));

        // assert that both objects are equal
        assert_eq!(root_manual, root_auto)
    }

    #[test]
    fn test_has_subfolder_exists_match() {
        // create two equal objects of the Folder struct
        let mut root = Folder::new("root".to_string());
        root.add_sub_folder(Folder::new("sub_1".to_string()), Path::new("root"));
        root.add_sub_folder(Folder::new("sub_2".to_string()), Path::new("root"));
        root.add_sub_folder(Folder::new("sub_sub_1".to_string()), Path::new("root/sub_1"));
        root.add_sub_folder(Folder::new("sub_sub_2".to_string()), Path::new("root/sub_1"));

        assert!(root.has_subfolder(Path::new("root/sub_1/sub_sub_2")))
    }

    #[test]
    fn test_has_subfolder_non_existent() {
        // create an object of the Folder struct
        let mut root = Folder::new("root".to_string());
        root.add_sub_folder(Folder::new("sub_1".to_string()), Path::new("root"));
        root.add_sub_folder(Folder::new("sub_2".to_string()), Path::new("root"));
        root.add_sub_folder(Folder::new("sub_sub_1".to_string()), Path::new("root/sub_1"));
        root.add_sub_folder(Folder::new("sub_sub_2".to_string()), Path::new("root/sub_1"));

        // assert that root does not have a sub folder 'sub_sub_4'
        assert_eq!(false, root.has_subfolder(Path::new("root/sub_1/sub_sub_4")))
    }

    #[test]
    fn test_has_subfolder_exists_no_match() {
        // create an object of the Folder struct
        let mut root = Folder::new("root".to_string());
        root.add_sub_folder(Folder::new("sub_1".to_string()), Path::new("root"));
        root.add_sub_folder(Folder::new("sub_2".to_string()), Path::new("root"));
        root.add_sub_folder(Folder::new("sub_sub_1".to_string()), Path::new("root/sub_1"));
        root.add_sub_folder(Folder::new("sub_sub_2".to_string()), Path::new("root/sub_1"));

        // assert that root does have a sub folder 'sub_sub_1' but not at 'root/sub_2/sub_sub_1'
        assert_eq!(false, root.has_subfolder(Path::new("root/sub_2/sub_sub_1")))
    }

}