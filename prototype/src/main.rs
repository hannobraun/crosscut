mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let game_io = language::start()?;
    game_io::start_and_wait(game_io)?;
    Ok(())
}
