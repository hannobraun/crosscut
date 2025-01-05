use std::{
    sync::mpsc::{self, SendError},
    thread,
};

use tuples::CombinRight;

pub struct Spawner<T> {
    actors: Option<T>,
}

impl Spawner<()> {
    pub fn new() -> Self {
        Self { actors: Some(()) }
    }
}

impl<T> Spawner<T> {
    pub fn spawn<I>(
        mut self,
        mut f: impl FnMut(I) -> bool + Send + 'static,
    ) -> (Spawner<T::Out>, Actor<I>)
    where
        T: CombinRight<()>,
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

        let Some(actors) = self.actors.take() else {
            unreachable!(
                "The field is only set to `None` right here, after which this \
                instance is dropped. Thus, it's not possible to encounter a \
                `None` value here."
            );
        };

        (
            Spawner {
                actors: Some(actors.push_right(())),
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
