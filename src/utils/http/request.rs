use std::collections::HashMap;

use http_types::headers::HeaderName;
use http_types::{Method, Url};

#[derive(Debug, Eq, PartialEq)]
pub struct Request {
    method: Method,
    url: Url,
    headers: HashMap<HeaderName, String>,
}

impl Request {
    pub fn new(method: Method, url: Url, headers: HashMap<HeaderName, String>) -> Self {
        Request {
            method,
            url,
            headers,
        }
    }
}

impl From<Request> for surf::Request {
    fn from(req: Request) -> Self {
        let mut new_req = surf::Request::new(req.method, req.url);

        req.headers.iter().for_each(|(k, v)| {
            new_req.insert_header(k, v.as_str());
        });

        new_req
    }
}
