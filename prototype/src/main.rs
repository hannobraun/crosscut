// This makes sense to prevent in public APIs, but it also warns me about the
// names of private modules that I only re-export from. In my opinion, it's too
// annoying for what little value it might provide.
#![allow(clippy::module_inception)]

mod editor;
mod game_engine;
mod io;
mod language;
mod thread;

fn main() -> anyhow::Result<()> {
    use game_engine::GameEngine;

    let (game_output_tx, game_output_rx) = thread::channel();

    let game_engine = GameEngine::start(game_output_tx)?;
    let editor = io::editor::input::start(game_engine.editor_input);
    io::game_engine::start_and_wait(game_engine.game_input, game_output_rx)?;

    game_engine.handle.join()?;
    editor.join()?;

    Ok(())
}
