pub mod opts;
pub mod rules;
pub mod stream_tweets;

pub use opts::{Opts, SubCmd};
pub use rules::{create_rule, delete_rule, delete_rules, get_rules, RULES_URL};
pub use stream_tweets::{stream_data, STREAM_URL, StreamResponse, StreamError};

use std::env;

pub fn get_bearer_token(opts: &Opts) -> String {
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

/// Gets the url using the tweet id: https://twitter.com/i/web/status/{id}
pub fn tweetid2url<T: ToString>(id: T) -> String {
    format!("https://twitter.com/i/web/status/{}", id.to_string())
}
