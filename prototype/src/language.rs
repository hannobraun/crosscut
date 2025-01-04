use tokio::sync::watch;

pub fn start_in_background() -> watch::Receiver<[f64; 4]> {
    let (_, color_rx) = watch::channel([0., 0., 0., 1.]);
    color_rx
}
