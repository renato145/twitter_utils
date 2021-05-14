use anyhow::{Context, Result};
use console::{Style, Term};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;

pub const STREAM_URL: &str = "https://api.twitter.com/2/tweets/search/stream";

pub async fn stream_data(out_file: &str, limit: Option<usize>, bearer_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let mut res = client
        .get(STREAM_URL)
        .header(header::AUTHORIZATION, bearer_token)
        .query(&[(
            "tweet.fields",
            "created_at,conversation_id,referenced_tweets,public_metrics,entities",
        )])
        .send()
        .await?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(out_file)?;

    let term = Term::stdout();
    let bold = Style::new().bold();
    println!("{}\n\n", bold.apply_to("Starting the stream..."));
    let mut processed = 0usize;
    let mut errors = 0usize;
    let mut finish = false;
    let green = Style::new().green();
    let red = Style::new().red();

    while let Some(chunk) = res.chunk().await.context("Error reading chunk")? {
        if chunk.len() < 10 {
            continue;
        }
        match serde_json::from_slice::<StreamResponse>(&chunk) {
            Ok(data) => {
                jsonl::write(&mut file, &data)?;
                processed += 1;
            }
            Err(e) => {
                eprintln!("Couldn't parse tweet data:\n{}\n{:?}", e, chunk);
                errors += 1;
            }
        }
        let mut progress = format!("{}", processed);
        if let Some(limit) = limit {
            progress.push_str(&format!("/{}", limit));
            if processed == limit {
                finish = true;
            }
        }
        term.clear_last_lines(2)?;
        println!("{} {}", green.apply_to("Processed tweets  :"), progress);
        println!("{} {}", red.apply_to("Errors encountered:"), errors);
        if finish {
            break;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub data: StreamResponseData,
    pub matching_rules: Option<Vec<RuleMatch>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponseData {
    pub id: String,
    pub text: String,
    pub created_at: String,
    pub conversation_id: String,
    #[serde(default)]
    pub referenced_tweets: Option<Vec<ReferencedTweets>>,
    pub public_metrics: PublicMetrics,
    pub entities: Option<Entities>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleMatch {
    pub id: usize,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferencedTweets {
    pub id: String,
    #[serde(rename = "type")]
    pub reference_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicMetrics {
    pub retweet_count: usize,
    pub reply_count: usize,
    pub like_count: usize,
    pub quote_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entities {
    pub annotations: Option<Vec<EntityAnnotation>>,
    pub urls: Option<Vec<EntityUrl>>,
    pub hashtags: Option<Vec<EntityTag>>,
    pub mentions: Option<Vec<EntityMention>>,
    pub cashtags: Option<Vec<EntityTag>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityAnnotation {
    pub start: usize,
    pub end: usize,
    pub probability: f32,
    #[serde(rename = "type")]
    pub annotation_type: String,
    pub normalized_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityUrl {
    pub start: usize,
    pub end: usize,
    pub url: String,
    pub expanded_url: String,
    pub display_url: String,
    pub unwound_url: Option<String>,
    pub images: Option<Vec<UrlImage>>,
    pub status: Option<usize>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlImage {
    pub url: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityTag {
    pub start: usize,
    pub end: usize,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityMention {
    pub start: usize,
    pub end: usize,
    pub username: String,
}
