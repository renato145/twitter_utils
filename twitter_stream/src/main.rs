use anyhow::Result;
use clap::{AppSettings, Clap};
use reqwest::{ Method, header };
use std::collections::HashMap;

/// Some app description
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// some arg
    #[clap(short, long)]
    arg: Option<String>,
    #[clap(subcommand)]
    subcmd: Option<SubCmd>,
}

#[derive(Clap, Debug)]
enum SubCmd {
    /// List current stream rules
    ListRules,
    CreateRule,
    DeleteRule,
}

type ResponseJSON = HashMap<String, String>;
const RULES_URL: &str = "https://api.twitter.com/2/tweets/search/stream/rules";

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let authorization = "";

    match opts.subcmd {
        Some(SubCmd::ListRules) => {
            // let client = reqwest::Client::new();
            // let res = client.request(Method::GET, RULES_URL).header(header::AUTHORIZATION, value)
            // let resp = reqwest::get(RULES_URL)
            //     .await?
            //     .json::<ResponseJSON>()
            //     .await?;
            // println!("{:?}: {:#?}", opts, resp);
        }
        Some(SubCmd::CreateRule) => {
            println!("create rule");
        }
        Some(SubCmd::DeleteRule) => {
            println!("delete rule");
        }
        None => {
            println!("main program here...");
        }
    }

    println!("{:#?}", opts);
    Ok(())
}
