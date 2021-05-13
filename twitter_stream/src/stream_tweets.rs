use anyhow::Result;
use reqwest::header;
use serde::Deserialize;

pub const STREAM_URL: &str = "https://api.twitter.com/2/tweets/search/stream";

pub async fn stream_data(bearer_token: &str) -> Result<()> {
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

#[derive(Debug, Deserialize)]
pub struct StreamResponse {
    pub data: StreamResponseData,
    pub matching_rules: Option<Vec<RuleMatch>>,
}

#[derive(Debug, Deserialize)]
pub struct RuleMatch {
    pub id: usize,
    pub tag: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamResponseData {
    pub id: String,
    pub text: String,
    pub created_at: String,
    pub conversation_id: String,
    pub public_metrics: PublicMetrics,
    pub entities: Entities,
}

#[derive(Debug, Deserialize)]
pub struct PublicMetrics {
    pub retweet_count: usize,
    pub reply_count: usize,
    pub like_count: usize,
    pub quote_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct Entities {
    pub annotations: Option<Vec<EntityAnnotation>>,
    pub urls: Option<Vec<EntityUrl>>,
    pub hashtags: Option<Vec<EntityTag>>,
    pub mentions: Option<Vec<EntityMention>>,
    pub cashtags: Option<Vec<EntityTag>>,
}

#[derive(Debug, Deserialize)]
pub struct EntityAnnotation {
    pub start: usize,
    pub end: usize,
    pub probability: f32,
    #[serde(rename(deserialize = "type"))]
    pub annotation_type: String,
    pub normalized_text: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct UrlImage {
    pub url: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Deserialize)]
pub struct EntityTag {
    pub start: usize,
    pub end: usize,
    pub tag: String,
}

#[derive(Debug, Deserialize)]
pub struct EntityMention {
    pub start: usize,
    pub end: usize,
    pub username: String,
}
