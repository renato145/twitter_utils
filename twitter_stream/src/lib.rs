pub mod opts;
pub mod rules;
pub mod stream_tweets;

pub use opts::{Opts, SubCmd};
pub use rules::{create_rule, delete_rule, delete_rules, get_rules, RULES_URL};
pub use stream_tweets::{stream_data, StreamError, StreamResponse, STREAM_URL};

use anyhow::{Context, Result};

pub fn get_bearer_token(bearer_token: Option<&str>, env_file: Option<&str>) -> Result<String> {
    let bearer_token = match bearer_token.clone() {
        Some(token) => token.to_string(),
        None => {
            if let Some(env_file) = env_file {
                dotenv::from_filename(env_file).ok();
            }
            std::env::var("BEARER_TOKEN")
                .context("$BEARER_TOKEN not found in enviroment variables, set the variable or specify it with --bearer_token")?
        }
    };
    Ok(format!("Bearer {}", bearer_token))
}

/// Gets the url using the tweet id: https://twitter.com/i/web/status/{id}
pub fn tweetid2url<T: ToString>(id: T) -> String {
    format!("https://twitter.com/i/web/status/{}", id.to_string())
}
