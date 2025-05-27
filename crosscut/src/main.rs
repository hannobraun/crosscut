fn main() -> anyhow::Result<()> {
    let threads = crosscut::threads::start()?;

    // This call is going to block until the user requests a shutdown via the
    // game I/O, or any of the other threads shut down.
    crosscut::io::game_engine::start_and_wait(
        threads.game_input,
        threads.game_output,
    )?;

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
