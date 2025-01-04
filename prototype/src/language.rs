use tokio::sync::watch;

pub fn start() -> watch::Receiver<wgpu::Color> {
    let (_, color_rx) = watch::channel(wgpu::Color::BLACK);
    color_rx
}
