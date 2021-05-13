use anyhow::Result;
use console::Style;
use std::{fs::File, io::BufReader};
use twitter_stream::{stream_tweets::StreamResponse, tweetid2url};

fn main() -> Result<()> {
    let file = File::open("twitter_data.jsonl")?;
    let mut reader = BufReader::new(file);
    let mut data = vec![];

    loop {
        match jsonl::read::<&mut BufReader<File>, StreamResponse>(&mut reader) {
            Ok(tweet) => {
                data.push(tweet);
            }
            Err(jsonl::ReadError::Eof) => {
                break;
            }
            _ => {}
        }
    }

    let n = 5;
    let bold = Style::new().bold();
    let msg = format!("{} tweets found", data.len());
    println!("{} (showing {} last tweets):", bold.apply_to(msg), n);
    let blue = bold.blue();

    data.iter().rev().take(n).for_each(|o| {
        let url = tweetid2url(&o.data.id);
        println!("- {}: \"{}\"", blue.apply_to(url), o.data.text);
    });

    Ok(())
}
