mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (color_tx, color_rx) = actor::channel();

    let (input, commands) = language::start(color_tx)?;
    let cli = cli::start(commands);
    game_io::start_and_wait(input, color_rx)?;

    cli.join()?;

    Ok(())
}
