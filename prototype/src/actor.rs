use std::{sync::mpsc, thread};

pub struct Actor<I> {
    pub input: Sender<I>,
}

pub fn actor<T>(mut f: impl FnMut(T) -> bool + Send + 'static) -> Actor<T>
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

    Actor { input: sender }
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
