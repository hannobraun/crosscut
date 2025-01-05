mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (color_tx, color_rx) = actor::channel();

    let (mut commands, mut input) = language::start(color_tx)?;
    let mut cli = cli::start(commands.sender);
    game_io::start_and_wait(input.sender, color_rx)?;

    commands.handle.join()?;
    input.handle.join()?;
    cli.join()?;

    Ok(())
}
