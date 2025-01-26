use std::path::PathBuf;

use super::Event;

#[tokio::test]
async fn basic_build() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let games_dir = PathBuf::from("../../games");
    let address = "[::1]:34481".parse()?;

    let mut events = crate::server::start(games_dir, address).await?;

    // Wait for server to be ready.
    while let Some(event) = events.recv().await {
        if let Event::ServerReady = event {
            break;
        }
    }

    reqwest::get("http://[::1]:34481/code").await?;

    Ok(())
}
