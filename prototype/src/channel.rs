use std::{sync::mpsc, thread};

pub fn actor<T>(mut f: impl FnMut(T) -> bool + Send + 'static) -> Sender<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = create();

    thread::spawn(move || {
        while let Ok(message) = receiver.recv() {
            if !f(message) {
                break;
            }
        }
    });

    sender
}

pub fn create<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::channel()
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
