mod nextcloud;

use nextcloud::NextcloudClient;

fn main() {
    println!("Hello, world!");
    let client = NextcloudClient::new("https://nextcloud.example.com");
    let _re = client.upload_file("/home/test/Converted/Steps.jpg");
}
