use hyper;
use hyper::header::{Cookie, Headers};
use cookie::CookieJar;

pub fn evimproved_headers(jar: Option<CookieJar>) -> Headers {
    let mut headers = Headers::new();
    let content_type = hyper::header::ContentType::form_url_encoded();
    headers.set(content_type);
    if let Some(jar) = jar {
        headers.set(Cookie::from_cookie_jar(&jar));
    }
    headers
}
