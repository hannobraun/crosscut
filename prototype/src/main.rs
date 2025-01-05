mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    use std::panic;

    let (color_tx, color_rx) = actor::channel();

    let (input, commands) = language::start(color_tx)?;
    let cli = cli::start(commands);
    game_io::start_and_wait(input, color_rx.into_inner())?;

    match cli.join() {
        Ok(()) => {}
        Err(payload) => {
            panic::resume_unwind(payload);
        }
    }

    Ok(())
}
