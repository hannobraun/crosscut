mod args;
mod build;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = args::Args::parse();
    let (source_code, bytecode) = build::build_snake().await?;
    let (_game_tx, game_rx) =
        tokio::sync::watch::channel((source_code, bytecode));
    server::start(args.address, args.serve_dir, game_rx).await?;

    tracing::info!("`capi-server` shutting down.");
    Ok(())
}
