use super::{Rule, RULES_URL};
use anyhow::{Context, Result};
use reqwest::header;
use serde::Deserialize;

pub async fn get_rules(bearer_token: &str) -> Result<ListRulesResponse> {
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

#[derive(Debug, Deserialize)]
pub struct ListRulesResponse {
    pub data: Option<Vec<Rule>>,
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
