use reqwest::{cookie::Jar, header as Header, Client, ClientBuilder, IntoUrl, RequestBuilder};
use std::sync::Arc;

pub enum Method {
    GET,
    POST,
}

pub fn get_default_headers() -> Header::HeaderMap {
    let mut headers = Header::HeaderMap::new();
    headers.insert("User-Agent", Header::HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4476.0 Safari/537.36"));
    headers
}

#[derive(Debug)]
pub struct ReqClient<'a> {
    jar: std::sync::Arc<Jar>,
    client: Client,
    cookie_file_path: Option<&'a str>,
}

impl<'a> ReqClient<'a> {
    pub fn new(custom_client: Option<&dyn Fn(ClientBuilder) -> ClientBuilder>) -> Self {
        let jar = std::sync::Arc::new(Jar::default());
        let j = jar.clone();
        let cli_builder = Client::builder()
            .default_headers(get_default_headers())
            .cookie_provider(jar);

        let cli_builder = match custom_client {
            Some(f) => f(cli_builder),
            None => cli_builder,
        };

        ReqClient {
            jar: j,
            client: cli_builder.build().unwrap(),
            cookie_file_path: None,
        }
    }

    // pub fn client(&self) -> &Client {
    //     &self.client
    // }

    pub fn cookie_jar(&self) -> &Arc<Jar> {
        &self.jar
    }

    pub fn set_cookie_file(&mut self, path: &'a str) {
        self.cookie_file_path = Some(path);
    }

    pub fn prepare<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req_builder = match method {
            Method::GET => self.client.get(url),
            Method::POST => self.client.post(url),
        };
        req_builder
    }
}
