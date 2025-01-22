// This makes sense to prevent in public APIs, but it also warns me about the
// names of private modules that I only re-export from. In my opinion, it's too
// annoying for what little value it might provide.
#![allow(clippy::module_inception)]

mod game_engine;
mod io;
mod lang;
mod threads;

fn main() -> anyhow::Result<()> {
    use crate::threads::GameEngineThread;

    let game_engine = GameEngineThread::start()?;

    io::game_engine::start_and_wait(
        game_engine.game_input,
        game_engine.game_output,
    )?;
    // At this point, we're blocking until any of the threads shut down. There's
    // nothing to join yet, because the game engine I/O is using `winit`
    // internally, which requires its own special handling.

    game_engine.handle.join()?;
    game_engine.editor_input.join()?;

    Ok(())
}
