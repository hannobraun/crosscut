// It makes sense to prevent this in public APIs, but it also warns me about the
// names of private modules that I only re-export from. That provides no value
// and is pretty annoying.
#![allow(clippy::module_inception)]

mod game_engine;
mod io;
mod language;
mod threads;

fn main() -> anyhow::Result<()> {
    let threads = threads::start()?;

    // This call is going to block until the user requests a shutdown via the
    // game I/O, or any of the other threads shut down.
    io::game_engine::start_and_wait(threads.game_input, threads.game_output)?;

    // At this point, the shutdown should be in progress, and none of these
    // calls block for long, if at all. The purpose of still joining all threads
    // is just to get any error they might have produced.
    for handle in threads.handles {
        handle.join()?;
    }

    Ok(())
}
