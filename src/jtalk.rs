use reqwest::Client;
use scraper::{ElementRef, Html, Selector};

#[derive(Debug)]
pub struct JTalk<'a> {
    csrf_token: Option<String>,
    logged_in: bool,
    account: Option<(&'a str, &'a str)>,
}

impl<'a> JTalk<'a> {
    pub fn new(email: &'a str, password: &'a str) -> Self {
        JTalk {
            csrf_token: None,
            logged_in: false,
            account: Some((email, password)),
        }
    }

    pub fn new_anonymous() -> Self {
        JTalk {
            csrf_token: None,
            logged_in: false,
            account: None,
        }
    }

    pub async fn login(&mut self) {}

    pub async fn refresh(&self) -> (String, bool) {
        let token: String;
        let mut logged_in: bool = false;
        let client = Client::new();
        let resp = client
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

    pub async fn get_token(&mut self) {}
}
