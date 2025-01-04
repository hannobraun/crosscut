mod application;
mod language;

fn main() -> anyhow::Result<()> {
    let updates = language::start_in_background();
    application::start_and_block(updates)?;
    Ok(())
}
