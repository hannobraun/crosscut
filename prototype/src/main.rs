mod game_engine;
mod io;
mod threads;

fn main() -> anyhow::Result<()> {
    let threads = threads::start()?;
    io::game_engine::start_and_wait(threads.game_input, threads.game_output)?;

    for handle in threads.handles {
        handle.join()?;
    }

    Ok(())
}
