use anyhow::Result;
use clap::{AppSettings, Clap};
use console::{Style, Term};
use elasticsearch::{http::transport::Transport, BulkOperation, BulkParts, Elasticsearch};
use jsonl::ReadError;
use serde_json::Value;
use std::{fs::File, io::BufReader};
use twitter_stream::StreamResponse;

/// Dumps the entire content of a JSON Lines file to Elastic Search
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// JSON Lines file
    jsonl_file: String,
    /// Batch size to send bulk messages to Elastic Search
    #[clap(short, long, default_value = "1000")]
    batch_size: usize,
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

    fn update_from_json(&mut self, json: Value) {
        if let Some(items) = json["items"].as_array() {
            let failed = items.iter().filter(|o| !o["error"].is_null()).count();
            let results = items
                .iter()
                .filter_map(|o| match &o["index"] {
                    Value::Object(index) => index.get("result").map(|o| o.as_str()).flatten(),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let created = results.iter().filter(|&&o| o == "created").count();
            let updated = results.iter().filter(|&&o| o == "updated").count();
            self.created += created;
            self.updated += updated;
            self.failed += failed;
        }
    }
}

struct BatchReader {
    reader: BufReader<File>,
    batch_size: usize,
    status: BatchReaderStatus,
    data: Vec<StreamResponse>,
}

impl BatchReader {
    fn new(reader: BufReader<File>, batch_size: usize) -> Self {
        Self {
            reader,
            batch_size,
            status: BatchReaderStatus::Reading,
            data: Vec::with_capacity(batch_size),
        }
    }

    fn read_batch(&mut self) -> &Vec<StreamResponse> {
        self.data.clear();

        loop {
            match jsonl::read::<&mut BufReader<File>, StreamResponse>(&mut self.reader) {
                Ok(tweet) => {
                    self.data.push(tweet);
                    if self.data.len() == self.batch_size {
                        break;
                    }
                }
                Err(ReadError::Eof) => {
                    self.status = BatchReaderStatus::Finished;
                    break;
                }
                _ => {}
            }
        }
        &self.data
    }
}

enum BatchReaderStatus {
    Reading,
    Finished,
}

/// https://www.elastic.co/guide/en/elasticsearch/reference/current/docs-bulk.html
async fn send_message_batch(
    batch: &[StreamResponse],
    client: &Elasticsearch,
    index: &str,
) -> Result<Value> {
    let body = batch
        .iter()
        .map(|o| BulkOperation::index(o).id(&o.data.id).into())
        .collect::<Vec<BulkOperation<_>>>();
    let response = client
        .bulk(BulkParts::Index(index))
        .body(body)
        .send()
        .await?;

    let json = response.json().await?;
    Ok(json)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let bold = Style::new().bold();
    let term = Term::stdout();

    let file = File::open(opts.jsonl_file)?;
    let reader = BufReader::new(file);
    let mut batch_reader = BatchReader::new(reader, opts.batch_size);

    println!("{}", bold.apply_to("Connecting to Elastic Search..."));
    let transport =
        Transport::single_node(&format!("http://{}:{}", opts.elastic_ip, opts.elastic_port))?;
    let client = Elasticsearch::new(transport);

    let mut summary = Summary::new();
    println!("{}", bold.apply_to("Start processing data..."));
    summary.show();

    loop {
        let batch = batch_reader.read_batch();
        let n = batch.len();
        match send_message_batch(batch, &client, &opts.elastic_index).await {
            Ok(json) => summary.update_from_json(json),
            Err(_err) => summary.failed += n,
        }
        term.clear_last_lines(3)?;
        summary.show();
        if let BatchReaderStatus::Finished = batch_reader.status {
            break;
        }
    }

    println!("{}", bold.apply_to("Done :)!"));
    Ok(())
}
