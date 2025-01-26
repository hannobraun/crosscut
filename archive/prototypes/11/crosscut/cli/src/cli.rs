use std::{fmt::Write, net::SocketAddr, path::PathBuf};

use anyhow::anyhow;
use clap::Parser;

use crate::{export::export, files, headless, server};

pub async fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    match args.command {
        Command::Export { path } => {
            check_files()?;
            export(args.games, path).await?;
        }
        Command::Headless => {
            headless::run(args.games).await?;
        }
        Command::Serve { address } => {
            check_files()?;

            let mut events = server::start(args.games, address).await?;

            while let Some(event) = events.recv().await {
                match event {
                    server::Event::ChangeDetected => {
                        print!(
                            "\n\
                            ⏳ Change detected. Building game...\n"
                        );
                    }
                    server::Event::BuildFinished => {
                        println!("✅ Finished building game.");
                    }
                    server::Event::ServerReady => {
                        print!(
                            "\n\
                            🚀 Build is ready: http://{address}/\n\
                            \n"
                        );
                    }
                }
            }

            tracing::info!("Crosscut server shutting down.");
        }
    }

    Ok(())
}

#[derive(clap::Parser)]
struct Args {
    #[arg(short, long, default_value = "games")]
    games: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Export {
        #[arg(short, long)]
        path: PathBuf,
    },
    Headless,
    Serve {
        /// Address to serve at
        #[arg(short, long, default_value = "127.0.0.1:34480")]
        address: SocketAddr,
    },
}

fn check_files() -> anyhow::Result<()> {
    let invalid_files = files::FILES.list_invalid();

    if invalid_files.is_empty() {
        return Ok(());
    }

    let mut err = String::new();

    write!(
        err,
        "Can't start the Crosscut tool because the following files are not \
        available:\n\
        \n",
    )?;

    for file in invalid_files {
        writeln!(err, "- `{file}`")?;
    }

    write!(
        err,
        "\n\
        All of those files should be included\n\
        with this tool! That they aren't, means\n\
        that the tool has not been built\n\
        correctly.\n\
        \n\
        Are you trying to run the server from\n\
        within the Crosscut repository?\n\
        \n\
        \tThen do so through the build tool!\n\
        \tJust execute `cargo run` from the\n\
        \trepository root.\n\
        \n\
        Are you trying to build a version of\n\
        this tool for use outside of the\n\
        repository?\n\
        \n\
        \tSorry, but as of 2024-11-23, this is\n\
        \tnot supported yet! If you're reading\n\
        \tthis some time after that date, then\n\
        \tmaybe it has become possible in the\n\
        \tmeantime, and this error message has not\n\
        \tbeen updated.\n",
    )?;

    Err(anyhow!("{}", err))
}
