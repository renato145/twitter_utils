use super::{RULES_URL, ResponseRuleMeta, Rule};
use anyhow::{anyhow, Context, Result};
use reqwest::header;
use serde::Deserialize;

pub async fn create_rule(rule: String, bearer_token: &str) -> Result<Rule> {
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

#[derive(Debug, Deserialize)]
pub struct CreateRuleResponse {
    pub data: Option<Vec<Rule>>,
    pub errors: Option<Vec<CreateRuleError>>,
    pub meta: ResponseRuleMeta,
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleError {
    pub value: String,
    pub details: Vec<String>,
    pub title: String,
    #[serde(rename(deserialize = "type"))]
    pub error_type: String,
}
