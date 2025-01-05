use std::sync::mpsc;

pub fn create<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::channel()
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
