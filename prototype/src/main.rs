mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (game_output_tx, color_rx) = actor::channel();

    let (mut language, mut commands, mut input) =
        language::start(game_output_tx)?;
    let mut cli = cli::start(commands.sender);
    game_io::start_and_wait(input.sender, color_rx)?;

    language.join()?;
    commands.handle.join()?;
    input.handle.join()?;
    cli.join()?;

    Ok(())
}
