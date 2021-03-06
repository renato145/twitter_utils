use anyhow::Result;
use clap::{AppSettings, Clap};
use console::{Style, Term};
use futures::StreamExt;
use twitter_stream::{get_bearer_token, stream_data, StreamError};

/// ZeroMQ publisher of Twitter stream
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Limits the number of tweets to process
    #[clap(short, long)]
    limit: Option<usize>,
    /// Token for twitter authentification, if not given the program
    /// will look for the environment variable BEARER_TOKEN.
    #[clap(short, long)]
    bearer_token: Option<String>,
    /// Enviroment file to look for $BEARER_TOKEN.
    #[clap(long, default_value = ".env")]
    env_file: String,
    /// Maximum number of connection resets while streaming
    #[clap(short, long)]
    max_resets: Option<usize>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// IP to bind the ZeroMQ socket
    #[clap(long, default_value = "127.0.0.1")]
    bind_ip: String,
    /// Port to bind the ZeroMQ socket
    #[clap(long, default_value = "5556")]
    bind_port: i32,
    /// If true ZeroMQ socket mode will be PUB otherwise PUSH is used
    /// (PUB does Fan out messages and PUSH Round-robin distribution of messages)
    #[clap(long)]
    socket_pub: bool,
    /// Envelope key used by the ZeroMQ publisher
    /// (used only for socket_pub=true)
    #[clap(short, long, default_value = "twitter_data")]
    envelope_key: String,
}

struct Summary {
    processed: usize,
    errors: usize,
    limit: Option<usize>,
    processed_style: Style,
    errors_style: Style,
}

impl Summary {
    fn new(limit: Option<usize>) -> Self {
        Self {
            processed: 0,
            errors: 0,
            limit,
            processed_style: Style::new().bold().green(),
            errors_style: Style::new().bold().red(),
        }
    }

    fn show(&self) {
        let mut processed = format!("{}", self.processed);
        if let Some(limit) = self.limit {
            processed.push_str(&format!("/{}", limit));
        }
        println!(
            "Processed tweets  : {}",
            self.processed_style.apply_to(processed)
        );
        println!(
            "Errors encountered: {}",
            self.errors_style.apply_to(self.errors)
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let term = Term::stdout();
    let bold = Style::new().bold();

    let bearer_token =
        get_bearer_token(opts.bearer_token.as_deref(), Some(opts.env_file.as_str()))?;

    let ctx = zmq::Context::new();
    let socket_type = if opts.socket_pub { zmq::PUB } else { zmq::PUSH };
    let publisher = ctx.socket(socket_type)?;
    publisher.bind(&format!("tcp://{}:{}", opts.bind_ip, opts.bind_port))?;

    let mut connection_resets = 0;
    let mut finish = false;

    let (mut rate_limit, mut stream) = stream_data(&bearer_token).await?;
    if opts.verbose > 0 {
        println!("{:?}", rate_limit);
    }
    let mut summary = Summary::new(opts.limit);
    println!("{}", bold.apply_to("Starting the stream..."));
    summary.show();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(tweet_data) => {
                if let Ok(msg) = serde_json::to_string(&tweet_data) {
                    if opts.socket_pub {
                        publisher
                            .send_multipart(&[&opts.envelope_key, &msg], 0)
                            .ok();
                    } else {
                        // push socket doesn't allow envelope filter
                        publisher.send(&msg, 0).ok();
                    }

                    summary.processed += 1;
                    if let Some(limit) = opts.limit {
                        if summary.processed == limit {
                            finish = true;
                        }
                    }

                    term.clear_last_lines(2)?;
                    summary.show();

                    if finish {
                        break;
                    }
                }
            }
            Err(StreamError::SmallChunk) => {}
            Err(StreamError::Parse(err)) => {
                eprintln!(
                    "Couldn't parse tweet data:\n{}\n{:?}\n\n",
                    err.source, err.msg
                );
                summary.errors += 1;
            }
            Err(StreamError::Reqwest(err)) => {
                // Try to reconnect
                if opts.verbose > 0 {
                    eprintln!("Error reading chunk of data: {:#?}", err);
                }
                summary.errors += 1;

                if let Some(max_resets) = opts.max_resets {
                    if connection_resets >= max_resets {
                        println!(
                            "Maximum number of connection resets ({}) reached...",
                            max_resets
                        );
                        break;
                    }
                }

                if let Some(rest) = rate_limit.duration_until_reset() {
                    println!("Waiting for rate limit ({:?})...", rest);
                    tokio::time::sleep(rest).await;
                    println!("Resetting connection...\n\n");
                }

                let (rl, s) = stream_data(&bearer_token).await?;

                connection_resets += 1;
                rate_limit = rl;
                stream = s;
            }
        }
    }

    Ok(())
}
