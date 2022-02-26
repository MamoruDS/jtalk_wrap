mod jtalk;
mod remap;
mod req;

use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;

use jtalk::JTalk;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Your input text
    text: String,

    /// File for saving cookies
    #[clap(long, value_name = "PATH")]
    cookie_file: Option<String>,

    /// JSON file for char remapping
    #[clap(short, long, value_name = "PATH")]
    remap: Option<String>,

    /// Remember option in j-talk login
    #[clap(long)]
    remember: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct Output {
    id: String,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_remap: Option<String>,
    logged_in: bool,
    result: Vec<jtalk::ConvertResult>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut j_cli = JTalk::new().remember(args.remember);

    let input = args.text;
    let input_remap = match args.remap {
        Some(fp) => {
            let map = std::fs::read_to_string(fp).unwrap_or(String::from("{}"));
            let map: serde_json::Value = serde_json::from_str(&map).unwrap();
            let mut cmap: HashMap<char, char> = HashMap::new();
            // TODO:
            for (k, v) in map.as_object().unwrap().iter() {
                let mut c = k.chars();
                let a = v.as_array().unwrap();
                let k: char = a.get(0).unwrap().as_str().unwrap().chars().nth(0).unwrap();
                cmap.insert(c.nth(0).unwrap(), k);
            }
            Some(remap::char_remap(&input, cmap))
        }
        None => None,
    };

    match args.cookie_file {
        Some(path) => j_cli.set_cookie_file(path),
        _ => {}
    }
    match (env::var("JTALK_EMAIL"), env::var("JTALK_PASSWD")) {
        (Ok(email), Ok(password)) => {
            j_cli.set_account(email, password);
        }
        _ => {}
    }
    j_cli.init().await;

    let (id, result) = j_cli
        .convert(match &input_remap {
            Some(text) => text,
            _ => &input,
        })
        .await;
    let output = Output {
        id,
        input,
        input_remap,
        logged_in: j_cli.is_logged_in(),
        result,
    };
    println!("{}", serde_json::to_string(&output).unwrap());
    Ok(())
}
