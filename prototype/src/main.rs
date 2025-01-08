mod actor;
mod editor;
mod game_io;
mod host;
mod language;
mod stdin;

fn main() -> anyhow::Result<()> {
    use host::Host;

    let (game_output_tx, game_output_rx) = actor::channel();

    let mut host = Host::start(game_output_tx)?;
    let mut editor = stdin::start(host.handle_editor_input.sender);
    game_io::start_and_wait(host.handle_game_input.sender, game_output_rx)?;

    host.handle.join()?;
    host.handle_editor_input.handle.join()?;
    host.handle_game_input.handle.join()?;
    editor.join()?;

    Ok(())
}
