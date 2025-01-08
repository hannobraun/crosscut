mod actor;
mod editor;
mod game_engine;
mod game_io;
mod language;
mod stdin;

fn main() -> anyhow::Result<()> {
    let (game_output_tx, game_output_rx) = actor::channel();

    let game_engine = game_engine::start(game_output_tx)?;
    let mut editor = stdin::start(game_engine.senders.editor_input);
    game_io::start_and_wait(game_engine.senders.game_input, game_output_rx)?;

    game_engine.threads.join()?;
    editor.join()?;

    Ok(())
}
