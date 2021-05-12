use anyhow::{anyhow, Context, Result};
use clap::{AppSettings, Clap};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

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
    DeleteRule(DeleteRule),
}

/// Creates a rule on the current stream
#[derive(Clap, Debug, Serialize, Deserialize)]
#[clap(
    after_help = "See: https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/integrate/build-a-rule"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct CreateRule {
    value: String,
    #[clap(short, long)]
    tag: Option<String>,
}

/// Delete a rule on the current stream
#[derive(Clap, Debug, Serialize, Deserialize)]
#[clap(
    after_help = "See: https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/api-reference/post-tweets-search-stream-rules#tab2"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct DeleteRule {
    id: Option<String>,
    #[clap(short, long)]
    all: bool,
    #[clap(short, long)]
    force: bool,
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
    data: Vec<Rule>,
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
struct CreateRuleResponse {
    data: Option<Vec<Rule>>,
    errors: Option<Vec<CreateRuleError>>,
    meta: ResponseRuleMeta,
}

#[derive(Debug, Deserialize)]
struct CreateRuleError {
    value: String,
    details: Vec<String>,
    title: String,
    #[serde(rename(deserialize = "type"))]
    error_type: String,
}

#[derive(Debug, Deserialize)]
struct DeleteRuleResponse {
    meta: ResponseRuleMeta,
}

#[derive(Debug, Deserialize)]
struct ResponseRuleMeta {
    summary: HashMap<String, usize>,
}

#[derive(Debug, Deserialize, Clone)]
struct Rule {
    id: String,
    value: String,
    tag: Option<String>,
}

impl std::fmt::Display for Rule {
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
    let res = client
        .get(RULES_URL)
        .header(header::AUTHORIZATION, bearer_token)
        .send()
        .await?
        .text()
        .await?;

    serde_json::from_str::<ListRulesResponse>(&res).with_context(|| {
        format!(
            "Couldn't parse response:\n{}",
            serde_json::to_string_pretty(&res).unwrap_or(res)
        )
    })
}

async fn create_rule(rule: String, bearer_token: &str) -> Result<Rule> {
    let client = reqwest::Client::new();
    let res = client
        .post(RULES_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, bearer_token)
        .body(format!("{{\"add\": [{}]}}", rule))
        .send()
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<CreateRuleResponse>(&res).with_context(|| {
        format!(
            "Couldn't parse response:\n{}",
            serde_json::to_string_pretty(&res).unwrap_or(res)
        )
    })?;

    if let Some(error) = res.errors {
        return Err(anyhow!("Error creating rule: {:#?}", error));
    }

    match &res.meta.summary.get("created") {
        Some(1) => Ok(res.data.unwrap()[0].clone()),
        _ => Err(anyhow!("Couldn't create rule: {:#?}", res)),
    }
}

async fn delete_rule(id: &str, bearer_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .post(RULES_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, bearer_token)
        .body(format!("{{\"delete\": {{ \"ids\": [ {:?} ] }} }}", id))
        .send()
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<DeleteRuleResponse>(&res).with_context(|| {
        format!(
            "Couldn't parse response:\n{}",
            serde_json::to_string_pretty(&res).unwrap_or(res)
        )
    })?;

    match &res.meta.summary.get("deleted") {
        Some(1) => Ok(()),
        _ => Err(anyhow!("Couldn't delete rule: {:#?}", res)),
    }
}

async fn delete_rules(ids: Vec<String>, bearer_token: &String) -> Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .post(RULES_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, bearer_token)
        .body(format!("{{\"delete\": {{ \"ids\": {:?} }} }}", ids))
        .send()
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<DeleteRuleResponse>(&res).with_context(|| {
        format!(
            "Couldn't parse response:\n{}",
            serde_json::to_string_pretty(&res).unwrap_or(res)
        )
    })?;

    let n = ids.len();
    match &res.meta.summary.get("deleted") {
        Some(&i) if i == n => Ok(()),
        _ => Err(anyhow!("Couldn't delete all the rules: {:#?}", res)),
    }
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
        Some(SubCmd::CreateRule(create_opts)) => {
            let rule_str =
                serde_json::to_string(&create_opts).context("Couldn't serialize rule")?;
            let rule = create_rule(rule_str, &bearer_token).await?;
            println!("{}", rule);
        }
        Some(SubCmd::DeleteRule(delete_opts)) => {
            if delete_opts.all {
                if delete_opts.force
                    || Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to delete all rules?")
                        .interact()
                        .unwrap()
                {
                    let ids = get_rules(&bearer_token)
                        .await?
                        .data
                        .into_iter()
                        .map(|o| o.id)
                        .collect::<Vec<_>>();

                    match ids.len() {
                        0 => println!("There are no rules in the stream"),
                        n => {
                            delete_rules(ids, &bearer_token).await?;
                            println!("All rules deleted: {}", n);
                        }
                    }
                }
                return Ok(());
            }

            let mut id = delete_opts.id;
            if let None = id {
                let rules = get_rules(&bearer_token).await?.data;
                id = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Pick the rule to delete")
                    .default(0)
                    .items(&rules)
                    .interact_opt()?
                    .map(|i| rules[i].id.clone());
            }

            if let Some(id) = id {
                delete_rule(&id, &bearer_token).await?;
                println!("Rule {:?} deleted", id);
            }
        }
        None => {
            println!("main program here...");
        }
    }

    Ok(())
}
