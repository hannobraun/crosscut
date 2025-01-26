mod game_engine;
mod io;
mod threads;

fn main() -> anyhow::Result<()> {
    let threads = threads::start()?;

    // This call is going to block until the user requests a shutdown via the
    // game I/O, or any of the other threads shut down.
    io::game_engine::start_and_wait(threads.game_input, threads.game_output)?;

    for handle in threads.handles {
        handle.join()?;
    }

    Ok(())
}
