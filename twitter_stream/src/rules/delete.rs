use super::{RULES_URL, ResponseRuleMeta};
use anyhow::{Context, Result, anyhow};
use reqwest::header;
use serde::Deserialize;

pub async fn delete_rule(id: &str, bearer_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let res = client
        .post(RULES_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, bearer_token)
        .body(format!("{{\"delete\": {{ \"ids\": [ {:?} ] }} }}", id))
        .send()
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<DeleteRuleResponse>(&res).with_context(|| {
        format!(
            "Couldn't parse response:\n{}",
            serde_json::to_string_pretty(&res).unwrap_or(res)
        )
    })?;

    match &res.meta.summary.get("deleted") {
        Some(1) => Ok(()),
        _ => Err(anyhow!("Couldn't delete rule: {:#?}", res)),
    }
}

pub async fn delete_rules(ids: Vec<String>, bearer_token: &str) -> Result<usize> {
    let client = reqwest::Client::new();
    let res = client
        .post(RULES_URL)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, bearer_token)
        .body(format!("{{\"delete\": {{ \"ids\": {:?} }} }}", ids))
        .send()
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<DeleteRuleResponse>(&res).with_context(|| {
        format!(
            "Couldn't parse response:\n{}",
            serde_json::to_string_pretty(&res).unwrap_or(res)
        )
    })?;

    let n = ids.len();
    match &res.meta.summary.get("deleted") {
        Some(&i) if i == n => Ok(n),
        _ => Err(anyhow!("Couldn't delete all the rules: {:#?}", res)),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteRuleResponse {
    pub meta: ResponseRuleMeta,
}
