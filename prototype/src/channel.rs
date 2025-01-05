use tokio::sync::mpsc;

pub fn create<T>() -> (Sender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

pub type Sender<T> = mpsc::UnboundedSender<T>;
