// This makes sense to prevent in public APIs, but it also warns me about the
// names of private modules that I only re-export from. In my opinion, it's too
// annoying for what little value it might provide.
#![allow(clippy::module_inception)]

mod game_engine;
mod io;
mod lang;
mod threads;

fn main() -> anyhow::Result<()> {
    let threads = threads::start()?;

    io::game_engine::start_and_wait(threads.game_input, threads.game_output)?;
    // At this point, we're blocking until any of the threads shut down. There's
    // nothing to join yet, because the game engine I/O is using `winit`
    // internally, which requires its own special handling.

    threads.handle.join()?;
    threads.editor_input.join()?;

    Ok(())
}
