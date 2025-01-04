mod application;

use tokio::sync::watch;

fn main() -> anyhow::Result<()> {
    let (_, color_rx) = watch::channel(wgpu::Color::BLACK);
    application::start(color_rx)?;
    Ok(())
}
