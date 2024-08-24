mod nextcloud;
mod media;
mod filesystem;

use filesystem::{File, Folder};
use nextcloud::NextcloudClient;
use std::env;
use std::path::Path;
use media::Extractor;

fn main() {
    let path = Path::new(".env");
    dotenv::from_path(path).expect("Failed to read .env file");

    // Helper function to retrieve environment variables
    fn get_env_var(var_name: &str) -> String {
        env::var(var_name).unwrap_or_else(|e| {
            eprintln!("Error while reading '{}': {}", var_name, e);
            String::new()
        })
    }

    let server_url = get_env_var("SERVER_URL");
    let username = get_env_var("NC_USERNAME");
    let password = get_env_var("PASSWORD");
    let exiftool = get_env_var("EXIFTOOL");

    //println!("{}:{}@{}", username, password, server_url);

    let client = NextcloudClient::new(server_url, username, password);

    let path = Path::new("/home/sealjonny/Github/nextsyncengine/Vineyard.jpg");
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        println!("file: {}", file_name);
    } else {
        println!("file: <default>");
    }
    
    let image = Path::new("/home/sealjonny/Github/nextsyncengine/Vineyard.jpg");

    let ext = Extractor::new(exiftool);

    let mut mtime: i64 = 0;
    if let Err(e) = ext.extract_date_time(image).map(|val| mtime = val) {
        eprintln!("{}", e);
        return
    }

    let mut file = File::new(image, Path::new("/TestHallo"));
    file.set_mtime(mtime);
    println!("{}", mtime);

    match client.is_online() {
        Ok(val) => println!("Is Client Online: {}", val),
        Err(e) => println!("Error while checking if server is online! {}", e)
    }
    
    match client.create_folder(Path::new("/TestHallo")) {
        Ok(_val) => print!("Successfully created demo folder"),
        Err(e) => {
            eprintln!("Error while creating folder: {}", e)
        }
    }
    match client.upload_file(file) {
        Ok(_val) => println!("Successfully uploaded demo!"),
        Err(e) => eprintln!("Error while uploading demo: {}", e)
    }

    match client.ls(Path::new("/Talk")) {
        Ok( val) =>  {
            for s in val {
                println!("{}", s)
            }
        }
        Err(e) => eprintln!("Error while listing Folder: {}", e)
    }

    let mut f = Folder::new("/Test".to_string());
    println!("{}", f.convert_to_string(0));
    f.add_sub_folder(Folder::new("Photos".to_string()), Path::new("/Test"));
    f.add_sub_folder(Folder::new("Test1".to_string()), Path::new("/Test/Photos"));
    f.add_sub_folder(Folder::new("Mara".to_string()), Path::new("/Test"));
    f.add_sub_folder(Folder::new("Liebe".to_string()), Path::new("/Test/Mara"));
    f.add_sub_folder(Folder::new("dich".to_string()), Path::new("/Test/Mara/Liebe"));
    println!("{}", f.convert_to_string(0));

    match client.exists_folder(Path::new("/TestHallo")) {
        Ok(val) => println!("{}", val),
        Err(e) => eprintln!("{}", e)
    }   
}
