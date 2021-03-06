use clap::{AppSettings, Clap};
use serde::{Deserialize, Serialize};

#[derive(Clap, Debug)]
#[clap(
    after_help = "See: https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/api-reference/get-tweets-search-stream"
)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// Limits the number of tweets to process
    #[clap(short, long)]
    pub limit: Option<usize>,
    /// File to store data
    #[clap(short, long, default_value = "twitter_data.jsonl")]
    pub file: String,
    /// Token for twitter authentification, if not given the program
    /// will look for the environment variable BEARER_TOKEN.
    #[clap(short, long)]
    pub bearer_token: Option<String>,
    /// Enviroment file to look for $BEARER_TOKEN.
    #[clap(long, default_value = ".env")]
    pub env_file: String,
    /// Maximum number of connection resets while streaming
    #[clap(short, long)]
    pub max_resets: Option<usize>,
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    #[clap(subcommand)]
    pub subcmd: Option<SubCmd>,
}

#[derive(Clap, Debug)]
pub enum SubCmd {
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
pub struct CreateRule {
    pub value: String,
    #[clap(short, long)]
    pub tag: Option<String>,
}

/// Delete a rule on the current stream
#[derive(Clap, Debug, Serialize, Deserialize)]
#[clap(
    after_help = "See: https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/api-reference/post-tweets-search-stream-rules#tab2"
)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct DeleteRule {
    pub id: Option<String>,
    #[clap(short, long)]
    pub all: bool,
    #[clap(short, long)]
    pub force: bool,
}
