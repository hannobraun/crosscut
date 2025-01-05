mod channel;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let (game_io, commands) = language::start()?;
    cli::start(commands);
    game_io::start_and_wait(game_io)?;
    Ok(())
}
