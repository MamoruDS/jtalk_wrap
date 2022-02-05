use reqwest::{header as Header, Client};

pub fn get_default_headers() -> Header::HeaderMap {
    let mut headers = Header::HeaderMap::new();
    headers.insert("User-Agent", Header::HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4476.0 Safari/537.36"));
    headers
}

#[derive(Debug)]
pub struct ReqClient {
    pub client: Client,
}

impl ReqClient {
    pub fn new() -> Self {
        ReqClient {
            client: Client::builder()
                .default_headers(get_default_headers())
                .cookie_store(true)
                .build()
                .unwrap(),
        }
    }
}
