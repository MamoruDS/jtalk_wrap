mod jtalk;
mod req;

use clap::Parser;
use serde_json::json;
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

    /// Remember option in j-talk login
    #[clap(long)]
    remember: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut j_cli = JTalk::new().remember(args.remember);

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

    let (id, result) = j_cli.convert(&args.text).await;
    println!(
        "{}",
        json!({"id":id, "logged_in": j_cli.is_logged_in(),"result":result})
    );
    Ok(())
}
