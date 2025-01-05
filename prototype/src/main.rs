mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    use std::sync::mpsc;
    let (color_tx, color_rx) = mpsc::channel();
    let (input, commands) = language::start(color_tx)?;
    cli::start(commands);
    game_io::start_and_wait(game_io::GameIo {
        input,
        output: color_rx,
    })?;
    Ok(())
}
