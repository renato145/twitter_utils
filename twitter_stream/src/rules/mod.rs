pub mod create;
pub mod delete;
pub mod get;

use std::collections::HashMap;

pub use create::create_rule;
pub use delete::{delete_rule, delete_rules};
pub use get::get_rules;

use serde::Deserialize;

pub const RULES_URL: &str = "https://api.twitter.com/2/tweets/search/stream/rules";

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub id: String,
    pub value: Option<String>,
    pub tag: Option<String>,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.value.as_ref().map(|o| o.as_str()).unwrap_or("");
        let mut out = format!("{}: {:?}", self.id, value);
        if let Some(tag) = &self.tag {
            out.push_str(&format!(" [tag: {:?}]", tag))
        }
        write!(f, "{}", out)
    }
}

#[derive(Debug, Deserialize)]
pub struct ResponseRuleMeta {
    pub summary: HashMap<String, usize>,
}
