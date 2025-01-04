mod application;
mod language;

fn main() -> anyhow::Result<()> {
    let color = language::start_in_background();
    application::start_and_block(color)?;
    Ok(())
}
