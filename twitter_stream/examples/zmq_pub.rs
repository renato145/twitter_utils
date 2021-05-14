use anyhow::Result;
use console::{Style, Term};
use futures::StreamExt;
use twitter_stream::{get_bearer_token, stream_data};

const ENVELOPE_KEY: &str = "twitter_data";

#[tokio::main]
async fn main() -> Result<()> {
    let bearer_token = get_bearer_token(None, Some(".env"))?;

    let ctx = zmq::Context::new();
    let publisher = ctx.socket(zmq::PUB)?;
    publisher.bind("tcp://127.0.0.1:5556")?;

    let bold = Style::new().bold();
    println!("{}", bold.apply_to("Server started"));
    println!("{}", bold.apply_to("Start streaming data..."));

    let mut i = 0;
    let term = Term::stdout();
    println!("Sended messages: {}", i);
    while let Some(Ok(chunk)) = stream_data(&bearer_token).await?.next().await {
        if let Ok(msg) = serde_json::to_string(&chunk) {
            publisher.send_multipart(&[ENVELOPE_KEY, &msg], 0).ok();
            i += 1;
            term.clear_last_lines(1)?;
            println!("Sended messages: {}", i);
        }
    }

    Ok(())
}
