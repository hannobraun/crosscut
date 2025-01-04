mod application;
mod language;

fn main() -> anyhow::Result<()> {
    let color_rx = language::start();
    application::start(color_rx)?;
    Ok(())
}
