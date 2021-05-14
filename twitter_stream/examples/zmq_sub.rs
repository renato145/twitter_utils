use anyhow::Result;
use console::Style;
use twitter_stream::{tweetid2url, StreamResponse};

const ENVELOPE_KEY: &str = "twitter_data";

fn main() -> Result<()> {
    let ctx = zmq::Context::new();
    let subscriber = ctx.socket(zmq::SUB)?;
    subscriber.connect("tcp://127.0.0.1:5556")?;
    subscriber.set_subscribe(ENVELOPE_KEY.as_bytes())?;

    let bold = Style::new().bold();
    let blue = Style::new().blue();
    println!("{}", bold.apply_to("Server started"));
    println!("{}", bold.apply_to("Start receiving data..."));

    loop {
        let _envelop = subscriber.recv_msg(0)?;
        let msg = subscriber.recv_bytes(0)?; //.unwrap();
        let tweet = serde_json::from_slice::<StreamResponse>(&msg)?;
        let url = tweetid2url(&tweet.data.id);
        println!("- {}: \"{}\"", blue.apply_to(url), tweet.data.text);
    }
}
