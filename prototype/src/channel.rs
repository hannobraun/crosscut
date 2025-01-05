use tokio::sync::mpsc;

pub fn create<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::unbounded_channel()
}

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;
