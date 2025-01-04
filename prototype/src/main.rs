mod game_io;
mod language;

fn main() -> anyhow::Result<()> {
    let updates = language::start_in_background();
    game_io::start_and_wait(updates)?;
    Ok(())
}
