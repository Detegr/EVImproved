use hyper;
use hyper::client::Client;
use hyper::header::Headers;
use urls::EVUrl;
use types::{Folder, FolderInfo};
use std::io::Read;
use url::form_urlencoded::serialize;

pub fn login(username: &str, password: &str) -> Option<Folder> {
    let data = serialize(&[
                   ("username", username),
                   ("password", password),
                   ("ajax", "true")
               ]);
    let mut client = Client::new();
    let mut headers = Headers::new();
    let content_type = hyper::header::ContentType::form_url_encoded();
    headers.set(content_type);
    match client.post(EVUrl::Login)
                .headers(headers)
                .body(&data)
                .send() {
        Ok(mut res) => {
            let mut ok = String::new();
            res.read_to_string(&mut ok);
            match &*ok {
                "TRUE" => {
                    println!("OK");
                },
                _ => {
                    println!("Not ok :(");
                }
            }
        },
        Err(e) => println!("Error: {}", e)
    };
    None
}
