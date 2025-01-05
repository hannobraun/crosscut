use std::{
    sync::mpsc::{self, SendError},
    thread::{self, JoinHandle},
};

pub struct Actor<I> {
    pub sender: Sender<I>,
    pub handle: JoinHandle<anyhow::Result<()>>,
}

impl<I> Actor<I> {
    pub fn spawn<F>(mut f: F) -> Actor<I>
    where
        I: Send + 'static,
        F: FnMut(I) -> Result<(), ChannelError> + Send + 'static,
    {
        let (sender, receiver) = channel();

        let handle = thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                match f(input) {
                    Ok(()) => {}
                    Err(ChannelError::Disconnected) => {
                        // Another actor has terminated. This means a shutdown
                        // is in progress and we should terminate too.
                        break;
                    }
                }
            }

            Ok(())
        });

        Actor { sender, handle }
    }

    pub fn provide_input<F>(self, mut f: F) -> JoinHandle<anyhow::Result<()>>
    where
        I: Send + 'static,
        F: FnMut() -> anyhow::Result<I> + Send + 'static,
    {
        thread::spawn(move || loop {
            let input = match f() {
                Ok(input) => input,
                Err(err) => {
                    panic!("{err:?}");
                }
            };

            if let Err(ChannelError::Disconnected) = self.sender.send(input) {
                break;
            }
        });

        self.handle
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = mpsc::channel();

    (Sender { inner: sender }, receiver)
}

pub struct Sender<T> {
    inner: mpsc::Sender<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), ChannelError> {
        self.inner
            .send(value)
            .map_err(|SendError(_)| ChannelError::Disconnected)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub type Receiver<T> = mpsc::Receiver<T>;

pub enum ChannelError {
    Disconnected,
}
