mod actor;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (input, color_rx, commands) = language::start()?;
    cli::start(commands);
    game_io::start_and_wait(game_io::GameIo {
        input,
        output: color_rx,
    })?;
    Ok(())
}
