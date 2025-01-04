mod application;
mod language;

fn main() -> anyhow::Result<()> {
    let color = language::start();
    application::start(color)?;
    Ok(())
}
