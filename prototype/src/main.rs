// This makes sense to prevent in public APIs, but it also warns me about the
// names of private modules that I only re-export from. In my opinion, it's too
// annoying for what little value it might provide.
#![allow(clippy::module_inception)]

mod game_engine;
mod io;
mod lang;
mod threads;

fn main() -> anyhow::Result<()> {
    use std::ops::ControlFlow;

    use crate::{game_engine::GameEngineThread, io::editor::input::read_event};

    let (game_output_tx, game_output_rx) = threads::channel();

    let game_engine = GameEngineThread::start(game_output_tx)?;
    let editor_input = threads::spawn(move || match read_event() {
        Ok(ControlFlow::Continue(event)) => {
            game_engine.editor_input.send(event)?;
            Ok(ControlFlow::Continue(()))
        }
        Ok(ControlFlow::Break(())) => Ok(ControlFlow::Break(())),
        Err(err) => Err(threads::Error::Other { err }),
    });

    io::game_engine::start_and_wait(game_engine.game_input, game_output_rx)?;
    // At this point, we're blocking until any of the threads shut down. There's
    // nothing to join yet, because the game engine I/O is using `winit`
    // internally, which requires its own special handling.

    game_engine.handle.join()?;
    editor_input.join()?;

    Ok(())
}
