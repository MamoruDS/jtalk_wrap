use crate::req::{Method, ReqClient};

use regex::Regex;
use reqwest::{
    cookie::CookieStore, header::LOCATION, redirect::Policy, ClientBuilder, StatusCode, Url,
};
use scraper::{ElementRef, Html, Selector};
use std::fs;

fn custom_client(client_builder: ClientBuilder) -> ClientBuilder {
    let custom_policy = Policy::custom(|attempt| {
        fn is_valid_redirect_path(path: &Vec<&str>) -> bool {
            if path.len() == 1 {
                match path.get(0).unwrap() {
                    &"contact" | &"convert" | &"logout" | &"register" => false,
                    _ => true,
                }
            } else {
                false
            }
        }
        let url = attempt.url();
        let path: Vec<&str> = url.path_segments().unwrap().collect();
        if attempt.previous().len() > 0 {
            match (url.host_str(), path, attempt.status()) {
                (Some("j-talk.com"), p, StatusCode::FOUND) if is_valid_redirect_path(&p) => {
                    return attempt.stop();
                }
                _ => {
                    // TODO: Err
                }
            }
        }
        attempt.follow()
    });
    client_builder.redirect(custom_policy)
}

pub async fn get_result(result_id: &str, req_client: &ReqClient) -> Vec<(String, Option<String>)> {
    let req = req_client.prepare(Method::GET, format!("https://j-talk.com/{}/raw", result_id));
    let resp = req.send().await.unwrap();
    let html = &resp.text().await.unwrap();
    let re = Regex::new(r"\n|<br>").unwrap();
    let html = format!("{}", re.replace_all(html, ""));
    let re = Regex::new(r"###\sBRACKETS(.+)</body>").unwrap();
    let caps = re.captures(&html).unwrap();
    let re = Regex::new(
        // using Han as delimiter
        // r"((?P<blk>[^\[|\]｜\p{Han}]{0,}(?P<bot>\p{Han}{1,})?)(\[(?P<top>[\p{Hiragana}|\p{Han}]+)\])?)",
        // using ^\[ as delimiter
        r"((?P<blk>[^\[|\]｜\p{Han}]{0,}(?P<bot>[^\[]{1,})?)(\[(?P<top>[\p{Hiragana}|\p{Han}]+)\])?)",
    )
    .unwrap();
    let mut results: Vec<(String, Option<String>)> = Vec::new();
    for cap in re.captures_iter(caps.get(1).map_or("", |m| m.as_str())) {
        let blk_char_vec = cap
            .name("blk")
            .unwrap()
            .as_str()
            .chars()
            .collect::<Vec<char>>();
        let blk_len = blk_char_vec.len();
        let bot_len = match cap.name("bot") {
            Some(m) => m.as_str().chars().count(),
            None => 0,
        };
        let top_text = match cap.name("top") {
            Some(m) => Some(String::from(m.as_str())),
            None => None,
        };
        if blk_len >= bot_len {
            let d = blk_len - bot_len;
            if d != 0 {
                results.push((blk_char_vec[..d].iter().collect(), None));
            }
            if d != blk_len {
                results.push((blk_char_vec[d..].iter().collect(), top_text));
            }
        }
    }
    results
}

#[derive(Debug)]
struct Config {
    pub account: Option<(String, String)>,
    pub remember: bool,
}

#[allow(dead_code)]
impl Config {
    pub fn has_account(&self) -> bool {
        match self.account {
            Some(_) => true,
            None => false,
        }
    }
    pub fn get_email(&self) -> &str {
        match &self.account {
            Some(acc) => &acc.0,
            _ => "",
        }
    }

    pub fn get_password(&self) -> &str {
        match &self.account {
            Some(acc) => &acc.1,
            _ => "",
        }
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct JTalk {
    req_cli: ReqClient,
    cookie_file_path: Option<String>,
    config: Config,
    csrf_token: Option<String>,
    logged_in: bool,
    _JTALK_URL: Url,
}

#[allow(dead_code)]
impl JTalk {
    pub fn new() -> Self {
        JTalk {
            req_cli: ReqClient::new(Some(&custom_client)),
            cookie_file_path: None,
            config: Config {
                account: None,
                remember: false,
            },
            csrf_token: None,
            logged_in: false,
            _JTALK_URL: Url::parse("https://j-talk.com/").unwrap(),
        }
    }

    pub fn remember(mut self, remember: bool) -> Self {
        self.config.remember = remember;
        self
    }

    pub fn has_account(&self) -> bool {
        self.config.has_account()
    }

    pub fn set_account(&mut self, email: String, password: String) {
        self.config.account = Some((email, password));
    }

    pub fn set_cookies(&self, cookie: String) {
        self.load_cookies(cookie);
    }

    pub fn set_cookie_file(&mut self, path: String) {
        self.cookie_file_path = Some(path);
        self.load_cookie_from_file();
    }

    pub async fn init(&mut self) {
        self.update().await;
        if !self.is_logged_in() && self.has_account() {
            self.login().await;
        }
    }

    pub fn load_cookies(&self, cookie: String) {
        let cookies = cookie.split('\n').collect::<Vec<&str>>();
        for cookie in cookies {
            self.req_cli
                .cookie_jar()
                .add_cookie_str(cookie, &self._JTALK_URL);
        }
    }

    fn save_cookie_to_file(&self) {
        if self.cookie_file_path.is_some() {
            match self.req_cli.cookie_jar().cookies(&self._JTALK_URL) {
                Some(header) => {
                    if !header.is_empty() {
                        let cookies = header
                            .to_str()
                            .unwrap()
                            .split(" ")
                            .collect::<Vec<&str>>()
                            .join("\n");
                        fs::write(self.cookie_file_path.as_ref().unwrap(), &cookies);
                    }
                }
                _ => {}
            }
        }
    }

    fn load_cookie_from_file(&self) {
        if self.cookie_file_path.is_some() {
            match fs::read_to_string(self.cookie_file_path.as_ref().unwrap()) {
                Ok(cookie_str) => {
                    self.load_cookies(cookie_str);
                }
                _ => {}
            }
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.logged_in
    }

    pub fn request_client(&self) -> &ReqClient {
        &self.req_cli
    }

    pub async fn login(&mut self) {
        let token: &str = &(self.get_token().await);

        if self.has_account() {
            let email = self.config.get_email();
            let password = self.config.get_password();
            let params = vec![
                ("_token", token),
                ("login", email),
                ("password", password),
                ("remember", &"on"),
            ];
            let params = if self.config.remember {
                &params
            } else {
                &params[..3]
            };
            let _ = self
                .req_cli
                .prepare(Method::POST, "https://j-talk.com/login")
                .form(params)
                .send()
                .await;
            self.update().await;
        }
    }

    // pub async fn logout(&mut self) {}

    async fn refresh(&self) -> (String, bool) {
        let token: String;
        let mut logged_in: bool = false;
        let resp = self
            .req_cli
            .prepare(Method::GET, "https://j-talk.com/convert")
            .send()
            .await
            .unwrap();
        let html = resp.text().await.unwrap();
        let fragment = Html::parse_fragment(&html);
        // csrf-token
        let selector = Selector::parse(r#"meta[name="csrf-token"]"#).unwrap();
        let selected = &fragment.select(&selector).collect::<Vec<ElementRef>>();
        assert_eq!(&1, &selected.len()); // TODO:
        token = String::from(selected.get(0).unwrap().value().attr("content").unwrap());
        // check login
        let selector = Selector::parse(".fa-rocket").unwrap();
        let selected = &fragment.select(&selector).collect::<Vec<ElementRef>>();
        match &selected.len() {
            1 => logged_in = true,
            _ => {}
        };
        self.save_cookie_to_file();
        (token, logged_in)
    }

    pub async fn update(&mut self) {
        let (token, logged_in) = self.refresh().await;
        self.csrf_token = Some(token);
        self.logged_in = logged_in;
    }

    pub async fn get_token(&mut self) -> String {
        let token = &self.csrf_token;
        match &token {
            Some(t) => {
                format!("{t}")
            }
            None => {
                self.update().await;
                format!("{}", &self.csrf_token.as_ref().unwrap())
            }
        }
    }

    pub async fn convert(&mut self, text: &str) -> (String, Vec<(String, Option<String>)>) {
        let token: &str = &(self.get_token().await);
        let params = [
            ("_token", token),
            ("content", text),
            ("convertOption", "main"),
        ];
        let resp = self
            .req_cli
            .prepare(Method::POST, "https://j-talk.com/convert")
            .form(&params)
            .send()
            .await
            .unwrap();
        let id = &resp.headers()[LOCATION];
        let url = Url::parse(id.to_str().unwrap()).unwrap();
        let split = url.path_segments().unwrap();
        let id = split.last().unwrap();
        self.save_cookie_to_file();
        (String::from(id), self.get_convert_result(id).await)
    }

    pub async fn get_convert_result(&self, result_id: &str) -> Vec<(String, Option<String>)> {
        get_result(result_id, &self.req_cli).await
    }
}
