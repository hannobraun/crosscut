mod actor;
mod cli;
mod code;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (game_output_tx, game_output_rx) = actor::channel();

    let (mut language, mut commands, mut game_input) =
        language::start(game_output_tx)?;
    let mut cli = cli::start(commands.sender);
    game_io::start_and_wait(game_input.sender, game_output_rx)?;

    language.join()?;
    commands.handle.join()?;
    game_input.handle.join()?;
    cli.join()?;

    Ok(())
}
