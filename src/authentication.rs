use hyper::client::Client;
use hyper::header::SetCookie;
use urls::EVUrl;
use types::{Folder, FolderId};
use std::io::Read;
use url::form_urlencoded::serialize;
use cookie::CookieJar;
use std::error::Error;
use headers::evimproved_headers;
use rustc_serialize::json;

pub fn login(username: &str, password: &str) -> Result<Folder, String> {
    let data = serialize(&[
                   ("username", username),
                   ("password", password),
                   ("ajax", "true")
               ]);
    let mut client = Client::new();
    let headers = evimproved_headers(None);
    let ret = match client.post(EVUrl::Login)
                .headers(headers)
                .body(&data)
                .send() {
        Ok(mut res) => {
            let mut ok = String::new();
            try!(res.read_to_string(&mut ok).map_err(|e| String::from(e.description())));
            match &*ok {
                "TRUE" => {
                    let mut jar = CookieJar::new(b"cookiejar");
                    let session_cookie = res.headers.get::<SetCookie>().unwrap(); // TODO: Result
                    session_cookie.apply_to_cookie_jar(&mut jar);
                    fetch_root_folder(jar)
                },
                _ => {
                    Err("Invalid username or password".into())
                }
            }
        },
        Err(e) => Err(String::from(e.description()))
    };
    ret
}

fn fetch_root_folder(jar: CookieJar) -> Result<Folder, String> {
    let mut client = Client::new();
    let headers = evimproved_headers(Some(jar));
    let ret = match client.get(EVUrl::Folder(FolderId::Root)).headers(headers.clone()).send() {
        Ok(mut res) => {
            let mut ok = String::new();
            try!(res.read_to_string(&mut ok).map_err(|e| String::from(e.description())));
            let mut folder: Folder = try!(json::decode(&ok).map_err(|e| String::from(e.description())));
            folder.set_headers(&headers);
            Ok(folder)
        },
        Err(e) => Err(String::from(e.description()))
    };
    ret
}
