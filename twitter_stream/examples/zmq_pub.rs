use anyhow::Result;
use futures::StreamExt;
use twitter_stream::{get_bearer_token, stream_data};

#[tokio::main]
async fn main() -> Result<()> {
    let bearer_token = get_bearer_token(None, Some(".env"))?;
    while let Some(Ok(chunk)) = stream_data(&bearer_token).await?.next().await {
        println!("{:?}", chunk);
    }

    Ok(())
}
