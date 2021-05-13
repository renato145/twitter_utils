use anyhow::{anyhow, Context, Result};
use clap::{AppSettings, Clap};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

const RULES_URL: &str = "https://api.twitter.com/2/tweets/search/stream/rules";
const STREAM_URL: &str = "https://api.twitter.com/2/tweets/search/stream";

/// Some app description
#[derive(Clap, Debug)]
#[clap(
    after_help = "See: https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/api-reference/get-tweets-search-stream"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Token for twitter authentification, if not given the program
    /// will look for the environment variable BEARER_TOKEN.
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
struct StreamResponse {
    data: StreamResponseData,
    matching_rules: Option<Vec<RuleMatch>>,
}

#[derive(Debug, Deserialize)]
struct RuleMatch {
    id: usize,
    tag: String,
}

#[derive(Debug, Deserialize)]
struct StreamResponseData {
    id: String,
    text: String,
    created_at: String,
    conversation_id: String,
    public_metrics: PublicMetrics,
    entities: Entities,
}

#[derive(Debug, Deserialize)]
struct PublicMetrics {
    retweet_count: usize,
    reply_count: usize,
    like_count: usize,
    quote_count: usize,
}

#[derive(Debug, Deserialize)]
struct Entities {
    annotations: Option<Vec<EntityAnnotation>>,
    urls: Option<Vec<EntityUrl>>,
    hashtags: Option<Vec<EntityTag>>,
    mentions: Option<Vec<EntityMention>>,
    cashtags: Option<Vec<EntityTag>>,
}

#[derive(Debug, Deserialize)]
struct EntityAnnotation {
    start: usize,
    end: usize,
    probability: f32,
    #[serde(rename(deserialize = "type"))]
    annotation_type: String,
    normalized_text: String,
}

#[derive(Debug, Deserialize)]
struct EntityUrl {
    start: usize,
    end: usize,
    url: String,
    expanded_url: String,
    display_url: String,
    unwound_url: Option<String>,
    images: Option<Vec<UrlImage>>,
    status: Option<usize>,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UrlImage {
    url: String,
    width: usize,
    height: usize,
}

#[derive(Debug, Deserialize)]
struct EntityTag {
    start: usize,
    end: usize,
    tag: String,
}

#[derive(Debug, Deserialize)]
struct EntityMention {
    start: usize,
    end: usize,
    username: String,
}

#[derive(Debug, Deserialize)]
struct ListRulesResponse {
    data: Option<Vec<Rule>>,
}

impl std::fmt::Display for ListRulesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self.data.as_ref() {
            Some(data) => {
                let mut out = format!("Found {} rules:", data.len());
                data.iter()
                    .for_each(|rule| out.push_str(format!("\n- {}", rule).as_str()));
                out
            }
            None => "No rules".into(),
        };
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
    value: Option<String>,
    tag: Option<String>,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value.as_ref().map(|o| o.as_str()).unwrap_or("");
        let mut out = format!("{}: {:?}", self.id, value);
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

async fn delete_rules(ids: Vec<String>, bearer_token: &str) -> Result<usize> {
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
        Some(&i) if i == n => Ok(n),
        _ => Err(anyhow!("Couldn't delete all the rules: {:#?}", res)),
    }
}

async fn stream_data(bearer_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let mut res = client
        .get(STREAM_URL)
        .header(header::AUTHORIZATION, bearer_token)
        .query(&[(
            "tweet.fields",
            "created_at,conversation_id,public_metrics,entities",
        )])
        .send()
        .await?;

    while let Some(chunk) = res.chunk().await? {
        match serde_json::from_slice::<StreamResponse>(&chunk) {
            Ok(data) => println!("{:#?}", data),
            Err(e) => eprintln!("Couldn't parse tweet data:\n{}\n{:?}", e, chunk),
        }
    }
    Ok(())
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
            // Delete all rules if --all
            if delete_opts.all {
                if delete_opts.force
                    || Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to delete all rules?")
                        .interact()
                        .unwrap()
                {
                    match get_rules(&bearer_token).await?.data {
                        Some(ids) => {
                            let ids = ids.into_iter().map(|o| o.id).collect::<Vec<_>>();
                            let n = delete_rules(ids, &bearer_token).await?;
                            println!("All rules deleted ({})", n);
                        }
                        None => {
                            println!("There are no rules in the stream")
                        }
                    }
                }
                return Ok(());
            }

            // Prompt select if no id was given
            let mut id = delete_opts.id;
            if id.is_none() {
                match get_rules(&bearer_token).await?.data {
                    Some(rules) => {
                        id = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Pick the rule to delete")
                            .default(0)
                            .items(&rules)
                            .interact_opt()?
                            .map(|i| rules[i].id.clone());
                    }
                    None => {
                        println!("There are no rules in the stream")
                    }
                }
            }

            // Delete 1 rule
            if let Some(id) = id {
                delete_rule(&id, &bearer_token).await?;
                println!("Rule {:?} deleted", id);
            }
        }
        None => {
            stream_data(&bearer_token).await?;
            println!("main program here...");
        }
    }

    Ok(())
}
