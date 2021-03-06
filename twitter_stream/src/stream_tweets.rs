use anyhow::Result;
use futures::{stream::IntoStream, Stream, StreamExt, TryStreamExt};
use reqwest::header::{self, HeaderMap};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::tweetid2url;

pub const STREAM_URL: &str = "https://api.twitter.com/2/tweets/search/stream";

pub async fn stream_data(
    bearer_token: &str,
) -> Result<(
    RateLimitHeaders,
    IntoStream<impl Stream<Item = std::result::Result<StreamResponse, StreamError>>>,
)> {
    let client = reqwest::Client::new();
    let res = client
        .get(STREAM_URL)
        .header(header::AUTHORIZATION, bearer_token)
        // https://developer.twitter.com/en/docs/twitter-api/tweets/filtered-stream/quick-start
        .query(&[
            (
                "tweet.fields",
                "created_at,conversation_id,referenced_tweets,public_metrics,entities",
            ),
            ("expansions", "author_id"),
            ("user.fields", "created_at"),
        ])
        .send()
        .await?;

    let rate_limit = RateLimitHeaders::from_headers(res.headers())?;

    let stream = res
        .bytes_stream()
        .map(|chunk| match chunk {
            Ok(chunk) => {
                if chunk.len() < 10 {
                    Err(StreamError::SmallChunk)
                } else {
                    // println!("{:?}\n\n", chunk);
                    serde_json::from_slice::<StreamResponse>(&chunk).map_err(|err| {
                        StreamError::Parse(ParseError {
                            msg: format!("{:?}\n\n", chunk),
                            source: err,
                        })
                    })
                }
            }
            Err(err) => Err(err.into()),
        })
        .into_stream();
    Ok((rate_limit, stream))
}

// "x-rate-limit-limit": "50",
// "x-rate-limit-reset": "1621007751",
// "x-rate-limit-remaining": "26",
#[derive(Debug)]
pub struct RateLimitHeaders {
    pub limit: Option<usize>,
    pub reset: Option<Duration>,
    pub remaining: Option<usize>,
}

impl RateLimitHeaders {
    pub fn from_headers(header_map: &HeaderMap) -> Result<Self> {
        let limit = match header_map.get("x-rate-limit-limit") {
            Some(o) => Some(o.to_str()?.parse()?),
            None => None,
        };
        let reset = match header_map.get("x-rate-limit-reset") {
            Some(o) => Some(Duration::from_secs(o.to_str()?.parse()?)),
            None => None,
        };
        let remaining = match header_map.get("x-rate-limit-remaining") {
            Some(o) => Some(o.to_str()?.parse()?),
            None => None,
        };
        Ok(RateLimitHeaders {
            limit,
            reset,
            remaining,
        })
    }

    // Get time until rate limit reset
    pub fn duration_until_reset(&self) -> Option<Duration> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH);
        if let (Some(0), Some(reset), Ok(now)) = (self.remaining, self.reset, now) {
            reset.checked_sub(now)
        } else {
            None
        }
    }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub data: StreamResponseData,
    pub includes: StreamResponseIncludes,
    pub matching_rules: Option<Vec<RuleMatch>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StreamResponseData {
    pub id: String,
    pub author_id: String,
    pub url: String,
    pub text: String,
    pub created_at: String,
    pub conversation_id: String,
    #[serde(default)]
    pub referenced_tweets: Option<Vec<ReferencedTweets>>,
    pub public_metrics: PublicMetrics,
    pub entities: Option<Entities>,
}

impl From<StreamResponseDataRaw> for StreamResponseData {
    fn from(raw: StreamResponseDataRaw) -> Self {
        let url = tweetid2url(&raw.id);
        Self {
            id: raw.id,
            author_id: raw.author_id,
            url,
            text: raw.text,
            created_at: raw.created_at,
            conversation_id: raw.conversation_id,
            referenced_tweets: raw.referenced_tweets,
            public_metrics: raw.public_metrics,
            entities: raw.entities,
        }
    }
}

impl<'de> Deserialize<'de> for StreamResponseData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let response: StreamResponseDataRaw = Deserialize::deserialize(deserializer)?;
        Ok(response.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamResponseDataRaw {
    pub id: String,
    pub author_id: String,
    pub url: Option<String>,
    pub text: String,
    pub created_at: String,
    pub conversation_id: String,
    #[serde(default)]
    pub referenced_tweets: Option<Vec<ReferencedTweets>>,
    pub public_metrics: PublicMetrics,
    pub entities: Option<Entities>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamResponseIncludes {
    pub users: Vec<UserData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub username: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleMatch {
    pub id: usize,
    pub tag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferencedTweets {
    pub id: String,
    #[serde(rename = "type")]
    pub reference_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicMetrics {
    pub retweet_count: usize,
    pub reply_count: usize,
    pub like_count: usize,
    pub quote_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entities {
    pub annotations: Option<Vec<EntityAnnotation>>,
    pub urls: Option<Vec<EntityUrl>>,
    pub hashtags: Option<Vec<EntityTag>>,
    pub mentions: Option<Vec<EntityMention>>,
    pub cashtags: Option<Vec<EntityTag>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityAnnotation {
    pub start: usize,
    pub end: usize,
    pub probability: f32,
    #[serde(rename = "type")]
    pub annotation_type: String,
    pub normalized_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UrlImage {
    pub url: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityTag {
    pub start: usize,
    pub end: usize,
    pub tag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityMention {
    pub start: usize,
    pub end: usize,
    pub username: String,
}
