// It makes sense to prevent this in public APIs, but it also warns me about the
// names of private modules that I only re-export from. That provides no value
// and is pretty annoying.
#![allow(clippy::module_inception)]

mod game_engine;
mod io;
mod language;
mod threads;
mod util;

pub use game_engine::{Game, PureCrosscutGame};

pub fn start_and_wait(game: Box<dyn Game + Send>) -> anyhow::Result<()> {
    let threads = threads::start()?;

    // This call is going to block until the user requests a shutdown via the
    // game I/O, or any of the other threads shut down.
    io::game_engine::start_and_wait(game, threads.terminal_input)?;

    // At this point, the shutdown should be in progress. None of these calls
    // should block for long, if at all. The purpose of still joining all
    // threads is just to get any error they might have produced.
    //
    // And let's join all threads first before printing any errors. Just to make
    // sure that they have ended, and the terminal is not still in raw mode or
    // something, when we start printing here.
    let results = threads.handles.map(|handle| handle.join());
    for result in results {
        result?;
    }

    Ok(())
}
