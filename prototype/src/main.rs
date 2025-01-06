mod actor;
mod code;
mod game_io;
mod language;
mod stdin;

fn main() -> anyhow::Result<()> {
    let (game_output_tx, game_output_rx) = actor::channel();

    let (mut language, mut commands, mut game_input) =
        language::start(game_output_tx)?;
    let mut editor = stdin::start(commands.sender);
    game_io::start_and_wait(game_input.sender, game_output_rx)?;

    language.join()?;
    commands.handle.join()?;
    game_input.handle.join()?;
    editor.join()?;

    Ok(())
}
