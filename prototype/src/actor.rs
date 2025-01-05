use std::{sync::mpsc, thread};

pub fn actor<T>(mut f: impl FnMut(T) -> bool + Send + 'static) -> Sender<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        while let Ok(input) = receiver.recv() {
            if !f(input) {
                break;
            }
        }
    });

    sender
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
