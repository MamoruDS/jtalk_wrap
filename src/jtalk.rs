use crate::req::ReqClient;

use regex::Regex;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

pub async fn get_result<'a>(result_id: &str, client: &Client) -> Vec<(String, Option<String>)> {
    let resp = client
        .get(format!("https://j-talk.com/{}/raw", result_id))
        .send()
        .await
        .unwrap();
    let html = &resp.text().await.unwrap();
    let re = Regex::new(r"\n|<br>").unwrap();
    let html = format!("{}", re.replace_all(html, ""));
    let re = Regex::new(r"###\sBRACKETS(.+)</body>").unwrap();
    let caps = re.captures(&html).unwrap();
    let re =
        Regex::new(r"(([^\[|\]｜\p{Han}]{0,}(\p{Han}{1,}))\[([\p{Hiragana}|\p{Han}]+)])").unwrap();
    let mut results: Vec<(String, Option<String>)> = Vec::new();
    for cap in re.captures_iter(caps.get(1).map_or("", |m| m.as_str())) {
        let blk_len = &cap[2].chars().count();
        let han_len = &cap[3].chars().count();
        let text = &cap[2];
        let hiragana = String::from(&cap[4]);
        match blk_len - han_len {
            0 => results.push((String::from(text), Some(hiragana))),
            d if d > 0 => {
                results.push((
                    text.chars().collect::<Vec<char>>()[..d].iter().collect(),
                    None,
                ));
                results.push((
                    text.chars().collect::<Vec<char>>()[d..].iter().collect(),
                    Some(hiragana),
                ))
            }
            _ => {
                // TODO:
                panic!("blk_len({}) smaller than han_len({})", blk_len, han_len)
            }
        };
    }
    results
}

#[derive(Debug)]
pub struct JTalk<'a> {
    req_cli: ReqClient,
    csrf_token: Option<String>,
    logged_in: bool,
    account: Option<(&'a str, &'a str)>,
}

impl<'a> JTalk<'a> {
    pub fn new(email: &'a str, password: &'a str) -> Self {
        JTalk {
            req_cli: ReqClient::new(),
            csrf_token: None,
            logged_in: false,
            account: Some((email, password)),
        }
    }

    pub fn new_anonymous() -> Self {
        JTalk {
            req_cli: ReqClient::new(),
            csrf_token: None,
            logged_in: false,
            account: None,
        }
    }

    fn client(&self) -> &Client {
        &self.req_cli.client
    }

    pub async fn login(&mut self) {}

    pub async fn refresh(&self) -> (String, bool) {
        let token: String;
        let mut logged_in: bool = false;
        let resp = self
            .client()
            .post("https://j-talk.com/convert")
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
}
