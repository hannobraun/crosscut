use std::{sync::mpsc, thread};

pub struct Actor<I> {
    pub input: Sender<I>,
}

impl<I> Actor<I> {
    pub fn start(mut f: impl FnMut(I) -> bool + Send + 'static) -> Actor<I>
    where
        I: Send + 'static,
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
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
