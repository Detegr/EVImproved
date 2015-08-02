use hyper::client::Client;
use hyper::header::SetCookie;
use urls::EVUrl;
use types::{EVError, Folder};
use std::io::Read;
use url::form_urlencoded::serialize;
use cookie::CookieJar;
use headers::evimproved_headers;

#[cfg(not(test))]
pub fn login(username: &str, password: &str) -> Result<Folder, EVError> {
    let data = serialize(&[
                   ("username", username),
                   ("password", password),
                   ("ajax", "true")
               ]);
    let headers = evimproved_headers(None);
    let client = Client::new();
    let ret = match client.post(EVUrl::Login)
                .headers(headers)
                .body(&data)
                .send() {
        Ok(mut res) => {
            let mut ok = String::new();
            try!(res.read_to_string(&mut ok));
            match &*ok {
                "TRUE" => {
                    let mut jar = CookieJar::new(b"cookiejar");
                    let session_cookie = res.headers.get::<SetCookie>().unwrap(); // TODO: Result
                    session_cookie.apply_to_cookie_jar(&mut jar);
                    Folder::fetch_root(jar)
                },
                _ => {
                    Err(EVError::Authentication("Invalid username or password".into()))
                }
            }
        },
        Err(e) => Err(EVError::from(e))
    };
    ret
}
