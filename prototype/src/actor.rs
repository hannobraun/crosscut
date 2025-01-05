use std::{
    sync::mpsc::{self, SendError},
    thread::{self, JoinHandle},
};

use tuples::CombinRight;

pub struct Spawner<T> {
    actors: T,
}

impl Spawner<()> {
    pub fn new() -> Self {
        Self { actors: () }
    }
}

impl<T> Spawner<T> {
    pub fn spawn<I>(
        self,
        mut f: impl FnMut(I) -> bool + Send + 'static,
    ) -> (Spawner<T::Out>, Actor<I>)
    where
        T: CombinRight<JoinHandle<()>>,
        I: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();

        let handle = thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                if !f(input) {
                    break;
                }
            }
        });

        (
            Spawner {
                actors: self.actors.push_right(handle),
            },
            Actor { sender },
        )
    }
}

pub struct Actor<I> {
    pub sender: Sender<I>,
}

impl<I> Actor<I> {
    pub fn provide_input(self, mut f: impl FnMut() -> I + Send + 'static)
    where
        I: Send + 'static,
    {
        thread::spawn(move || loop {
            let input = f();

            if let Err(SendError(_)) = self.sender.send(input) {
                break;
            }
        });
    }
}

pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
