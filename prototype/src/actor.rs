use std::{
    sync::mpsc::{self, SendError},
    thread,
};

pub struct Actor<I> {
    pub input: Sender<I>,
}

impl<I> Actor<I> {
    pub fn start(mut f: impl FnMut(I) -> bool + Send + 'static) -> Self
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

        Self { input: sender }
    }

    pub fn provide_input(self, mut f: impl FnMut() -> I + Send + 'static)
    where
        I: Send + 'static,
    {
        thread::spawn(move || loop {
            let input = f();

            if let Err(SendError(_)) = self.input.send(input) {
                break;
            }
        });
    }
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
