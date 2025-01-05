mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let game_io = language::start_in_background()?;
    game_io::start_and_wait(game_io)?;
    Ok(())
}
