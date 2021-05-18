use anyhow::Result;
use clap::{AppSettings, Clap};
use console::{Style, Term};
use elasticsearch::{http::transport::Transport, Elasticsearch, IndexParts};
use serde_json::Value;
use std::{fs::File, io::BufReader};
use twitter_stream::StreamResponse;

/// Dumps the entire content of a JSON Lines file to Elastic Search
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// JSON Lines file
    jsonl_file: String,
    /// IP for the elastic search instance
    #[clap(long, default_value = "127.0.0.1")]
    elastic_ip: String,
    /// Port for the elastic search instance
    #[clap(long, default_value = "9200")]
    elastic_port: i32,
    /// Index to use for elastic search
    #[clap(long, default_value = "tweets")]
    elastic_index: String,
}

/// https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-index_.html
#[derive(Debug)]
enum ESResponse {
    Created,
    Updated,
    Failed,
}

async fn send_message(
    msg: StreamResponse,
    client: &Elasticsearch,
    index: &str,
) -> Result<ESResponse> {
    let response = client
        .index(IndexParts::IndexId(index, &msg.data.id))
        .body(&msg)
        .send()
        .await?;
    if response.status_code().is_success() {
        let response: Value = response.json().await?;
        let result = match response["result"].as_str() {
            Some("created") => ESResponse::Created,
            Some("updated") => ESResponse::Updated,
            _ => ESResponse::Failed,
        };
        Ok(result)
    } else {
        Ok(ESResponse::Failed)
    }
}

struct Summary {
    created: usize,
    updated: usize,
    failed: usize,
    created_style: Style,
    updated_style: Style,
    failed_style: Style,
}

impl Summary {
    fn new() -> Self {
        Self {
            created: 0,
            updated: 0,
            failed: 0,
            created_style: Style::new().bold().green(),
            updated_style: Style::new().bold().blue(),
            failed_style: Style::new().bold().red(),
        }
    }

    fn show(&self) {
        println!("Created: {}", self.created_style.apply_to(self.created));
        println!("Updated: {}", self.updated_style.apply_to(self.updated));
        println!("Failed : {}", self.failed_style.apply_to(self.failed));
    }

    fn update(&mut self, response: ESResponse) {
        match response {
            ESResponse::Created => self.created += 1,
            ESResponse::Updated => self.updated += 1,
            ESResponse::Failed => self.failed += 1,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let bold = Style::new().bold();
    let term = Term::stdout();

    let file = File::open(opts.jsonl_file)?;
    let mut reader = BufReader::new(file);

    println!("{}", bold.apply_to("Connecting to Elastic Search..."));
    let transport =
        Transport::single_node(&format!("http://{}:{}", opts.elastic_ip, opts.elastic_port))?;
    let client = Elasticsearch::new(transport);

    let mut summary = Summary::new();
    println!("{}", bold.apply_to("Start processing data..."));
    summary.show();

    loop {
        match jsonl::read::<&mut BufReader<File>, StreamResponse>(&mut reader) {
            Ok(tweet) => match send_message(tweet, &client, &opts.elastic_index).await {
                Ok(res) => summary.update(res),
                Err(_err) => summary.failed += 1,
            },
            Err(jsonl::ReadError::Eof) => {
                break;
            }
            _ => summary.failed += 1,
        }
        term.clear_last_lines(3)?;
        summary.show();
    }

    Ok(())
}
