use anyhow::Result;
use console::Style;
use std::{fs::File, io::BufReader};
use twitter_stream::stream_tweets::StreamResponse;

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

    let bold = Style::new().bold();
    let msg = format!("{} tweets found:", data.len());
    println!("{}", bold.apply_to(msg));

    data.iter()
        .take(5)
        .for_each(|o| println!("- {}", o.data.text));

    Ok(())
}
