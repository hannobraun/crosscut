mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (color_tx, color_rx) = actor::channel();

    let (mut input, mut commands) = language::start(color_tx)?;
    let mut cli = cli::start(commands.sender);
    game_io::start_and_wait(input.sender, color_rx)?;

    input.handle.join()?;
    commands.handle.join()?;
    cli.join()?;

    Ok(())
}
