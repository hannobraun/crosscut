mod actor;
mod editor;
mod game_io;
mod host;
mod language;
mod stdin;

fn main() -> anyhow::Result<()> {
    let (game_output_tx, game_output_rx) = actor::channel();

    let (mut host, mut handle_editor_input, mut game_input) =
        host::start(game_output_tx)?;
    let mut editor = stdin::start(handle_editor_input.sender);
    game_io::start_and_wait(game_input.sender, game_output_rx)?;

    host.join()?;
    handle_editor_input.handle.join()?;
    game_input.handle.join()?;
    editor.join()?;

    Ok(())
}
