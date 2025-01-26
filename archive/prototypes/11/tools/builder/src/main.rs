mod build;
mod export;
mod serve;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use anyhow::Context;
    use clap::Parser;

    tracing_subscriber::fmt().init();

    let args = Args::parse();
    match args.command {
        Some(Command::Export) => {
            export::run().await.context("Running `export` command")?;
        }
        None | Some(Command::Serve) => {
            serve::start().await.context("Running `serve` command")?;
        }
    }

    Ok(())
}

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    Export,
    Serve,
}
