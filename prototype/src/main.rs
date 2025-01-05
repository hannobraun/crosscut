mod channel;
mod cli;
mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let commands = cli::start();
    let game_io = language::start(commands)?;
    game_io::start_and_wait(game_io)?;
    Ok(())
}
