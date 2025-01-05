use tokio::sync::mpsc;

pub fn create<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}
