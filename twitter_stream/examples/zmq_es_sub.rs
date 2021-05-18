use anyhow::Result;
use clap::{AppSettings, Clap};
use console::{Style, Term};
use elasticsearch::{http::transport::Transport, Elasticsearch, IndexParts};
use serde_json::Value;
use twitter_stream::StreamResponse;

/// ZeroMQ Elastic Search Subscriber
/// Gets messages from the publisher and save them to Elastic Search
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// IP for the elastic search instance
    #[clap(long, default_value = "127.0.0.1")]
    elastic_ip: String,
    /// Port for the elastic search instance
    #[clap(long, default_value = "9200")]
    elastic_port: i32,
    /// Index to use for elastic search
    #[clap(long, default_value = "tweets")]
    elastic_index: String,
    /// IP to bind the ZeroMQ socket
    #[clap(long, default_value = "127.0.0.1")]
    bind_ip: String,
    /// Port to bind the ZeroMQ socket
    #[clap(long, default_value = "5556")]
    bind_port: i32,
    /// Envelope key used by the ZeroMQ publisher
    #[clap(short, long, default_value = "twitter_data")]
    envelope_key: String,
}

fn get_message(subscriber: &zmq::Socket) -> Result<StreamResponse> {
    let _envelop = subscriber.recv_msg(0)?;
    let msg = subscriber.recv_bytes(0)?;
    serde_json::from_slice::<StreamResponse>(&msg).map_err(|err| err.into())
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

    println!("{}", bold.apply_to("Connecting to Elastic Search..."));
    let transport =
        Transport::single_node(&format!("http://{}:{}", opts.elastic_ip, opts.elastic_port))?;
    let client = Elasticsearch::new(transport);

    println!("{}", bold.apply_to("Connecting to ZeroMQ..."));
    let ctx = zmq::Context::new();
    let subscriber = ctx.socket(zmq::SUB)?;
    subscriber.connect("tcp://127.0.0.1:5556")?;
    subscriber.set_subscribe(opts.envelope_key.as_bytes())?;

    term.clear_last_lines(2)?;
    let mut summary = Summary::new();
    println!("{}", bold.apply_to("Start receiving data..."));
    summary.show();

    loop {
        let msg = get_message(&subscriber)?;
        match send_message(msg, &client, &opts.elastic_index).await {
            Ok(res) => summary.update(res),
            Err(_err) => summary.failed += 1,
        }
        term.clear_last_lines(3)?;
        summary.show();
    }
}
