use std::{fs::OpenOptions, io::Write};

use anyhow::Result;
use reqwest::header;
use serde::{Deserialize, Serialize};

pub const STREAM_URL: &str = "https://api.twitter.com/2/tweets/search/stream";

pub async fn stream_data(out_file: &str, bearer_token: &str) -> Result<()> {
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

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(out_file)?;

    while let Some(chunk) = res.chunk().await? {
        match serde_json::from_slice::<StreamResponse>(&chunk) {
            Ok(data) => {
                jsonl::write(&mut file, &data)?;
                file.flush()?;
            }
            Err(e) => eprintln!("Couldn't parse tweet data:\n{}\n{:?}", e, chunk),
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
pub struct RuleMatch {
    pub id: usize,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponseData {
    /// To retrieve the url use https://twitter.com/i/web/status/{id}
    pub id: String,
    pub text: String,
    pub created_at: String,
    pub conversation_id: String,
    pub public_metrics: PublicMetrics,
    pub entities: Entities,
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
    #[serde(rename(deserialize = "type"))]
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
