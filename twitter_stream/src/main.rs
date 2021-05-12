use anyhow::{Context, Result};
use clap::{AppSettings, Clap};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::env;

/// Some app description
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Token for twitter authentification, if not given it will look
    /// for the environment variable BEARER_TOKEN.
    #[clap(short, long)]
    bearer_token: Option<String>,
    /// Enviroment file to look for $BEARER_TOKEN.
    #[clap(long, default_value = ".env")]
    env_file: String,
    #[clap(subcommand)]
    subcmd: Option<SubCmd>,
}

#[derive(Clap, Debug)]
enum SubCmd {
    /// List current stream rules
    ListRules,
    CreateRule(CreateRule),
    DeleteRule,
}

/// Creates a rule on the current stream
/// https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/integrate/build-a-rule
#[derive(Clap, Debug, Serialize, Deserialize)]
struct CreateRule {
    value: String,
    #[clap(short, long)]
    tag: Option<String>,
}

const RULES_URL: &str = "https://api.twitter.com/2/tweets/search/stream/rules";

fn get_bearer_token(opts: &Opts) -> String {
    let bearer_token = match opts.bearer_token.clone() {
        Some(token) => token,
        None => {
            dotenv::from_filename(&opts.env_file).ok();
            env::var("BEARER_TOKEN")
                .expect("$BEARER_TOKEN not found in enviroment variables, set the variable or specify it with --bearer_token")
        }
    };
    format!("Bearer {}", bearer_token)
}

#[derive(Debug, Deserialize)]
struct ListRulesResponse {
    data: Vec<RuleResponse>,
}

impl std::fmt::Display for ListRulesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!("Found {} rules:", self.data.len());
        self.data
            .iter()
            .for_each(|rule| out.push_str(format!("\n- {}", rule).as_str()));
        write!(f, "{}", out)
    }
}

#[derive(Debug, Deserialize)]
struct RuleResponse {
    id: String,
    value: String,
    tag: Option<String>,
}

impl std::fmt::Display for RuleResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!("{}: {:?}", self.id, self.value);
        if let Some(tag) = &self.tag {
            out.push_str(&format!(" [tag: {:?}]", tag))
        }
        write!(f, "{}", out)
    }
}

async fn get_rules(bearer_token: &str) -> Result<ListRulesResponse> {
    let client = reqwest::Client::new();
    client
        .get(RULES_URL)
        .header(header::AUTHORIZATION, bearer_token)
        .send()
        .await?
        .json::<ListRulesResponse>()
        .await
        .context("Couldn't parse reponse")
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let bearer_token = get_bearer_token(&opts);

    match opts.subcmd {
        Some(SubCmd::ListRules) => {
            let rules = get_rules(&bearer_token).await?;
            println!("{}", rules);
        }
        Some(SubCmd::CreateRule(rule)) => {
            let rule = serde_json::to_string(&rule).context("Couldn't serialize rule")?;
            let client = reqwest::Client::new();
            let result = client
                .post(RULES_URL)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, bearer_token)
                .body(format!("{{\"add\": [{}]}}", rule))
                .send()
                .await?
                .text()
                .await?;
            println!("{}", result);
        }
        Some(SubCmd::DeleteRule) => {
            println!("delete rule");
        }
        None => {
            println!("main program here...");
        }
    }

    // println!("{:#?}", opts);
    Ok(())
}
