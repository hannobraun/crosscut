use std::{
    sync::mpsc::{self, SendError},
    thread::{self, JoinHandle},
};

pub struct Actor<I> {
    pub sender: Sender<I>,
    pub handle: JoinHandle<()>,
}

impl<I> Actor<I> {
    pub fn spawn(mut f: impl FnMut(I) -> bool + Send + 'static) -> Actor<I>
    where
        I: Send + 'static,
    {
        let (sender, receiver) = channel();

        let handle = thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                if !f(input) {
                    break;
                }
            }
        });

        Actor { sender, handle }
    }

    pub fn provide_input(
        self,
        mut f: impl FnMut() -> I + Send + 'static,
    ) -> JoinHandle<()>
    where
        I: Send + 'static,
    {
        thread::spawn(move || loop {
            let input = f();

            if let Err(SendError(_)) = self.sender.send(input) {
                break;
            }
        });

        self.handle
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::channel()
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
