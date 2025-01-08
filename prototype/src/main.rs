mod actor;
mod editor;
mod game_engine;
mod game_io;
mod language;
mod stdin;

fn main() -> anyhow::Result<()> {
    let (game_output_tx, game_output_rx) = actor::channel();

    let host = game_engine::start(game_output_tx)?;
    let mut editor = stdin::start(host.senders.editor_input);
    game_io::start_and_wait(host.senders.game_input, game_output_rx)?;

    host.threads.join()?;
    editor.join()?;

    Ok(())
}
