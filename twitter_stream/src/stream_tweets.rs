use anyhow::Result;
use futures::{stream::IntoStream, Stream, StreamExt, TryStreamExt};
use reqwest::header;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const STREAM_URL: &str = "https://api.twitter.com/2/tweets/search/stream";

pub async fn stream_data(
    bearer_token: &str,
) -> Result<IntoStream<impl Stream<Item = std::result::Result<StreamResponse, StreamError>>>> {
    let client = reqwest::Client::new();
    let res = client
        .get(STREAM_URL)
        .header(header::AUTHORIZATION, bearer_token)
        .query(&[(
            "tweet.fields",
            "created_at,conversation_id,referenced_tweets,public_metrics,entities",
        )])
        .send()
        .await?;

    // TODO: maybe take care of rate limits
    // "x-rate-limit-limit": "50",
    // "x-rate-limit-reset": "1621007751",
    // "x-rate-limit-remaining": "26",
    // let headers = res.headers();
    // let rate_limit = headers.get("x-rate-limit-limit");
    // let rate_limit_reset = headers.get("x-rate-limit-reset");
    // let rate_limit_remaining = headers.get("x-rate-limit-remaining");

    let stream = res
        .bytes_stream()
        .into_stream()
        .map(|chunk| match chunk {
            Ok(chunk) => {
                if chunk.len() < 10 {
                    Err(StreamError::SmallChunk)
                } else {
                    serde_json::from_slice::<StreamResponse>(&chunk).map_err(|err| {
                        StreamError::Parse(ParseError {
                            msg: format!("{:?}", chunk),
                            source: err,
                        })
                    })
                }
            }
            Err(err) => Err(err.into()),
        })
        .into_stream();
    Ok(stream)
}

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("The readed chunk is too small to parse")]
    SmallChunk,
    #[error("Error reading chunk of stream")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Parse(ParseError),
}

#[derive(Error, Debug)]
#[error("Error parsing tweet data:\n{source}\n{msg}")]
pub struct ParseError {
    pub msg: String,
    pub source: serde_json::Error,
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
