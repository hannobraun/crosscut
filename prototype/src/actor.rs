use std::{
    panic,
    sync::mpsc::{self, SendError},
    thread::{self, JoinHandle},
};

pub struct Actor<I> {
    pub sender: Sender<I>,
    pub handle: ThreadHandle,
}

impl<I> Actor<I> {
    pub fn spawn<F>(mut f: F) -> Actor<I>
    where
        I: Send + 'static,
        F: FnMut(I) -> Result<(), Error> + Send + 'static,
    {
        let (sender, receiver) = channel();

        let handle = thread::spawn(move || {
            while let Ok(input) = receiver.recv() {
                match f(input) {
                    Ok(()) => {}
                    Err(Error::ChannelDisconnected) => {
                        break;
                    }
                    Err(Error::Other { err }) => {
                        return Err(err);
                    }
                }
            }

            Ok(())
        });

        Actor {
            sender,
            handle: ThreadHandle::new(handle),
        }
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
    pub fn send(&self, value: T) -> Result<(), Error> {
        self.inner
            .send(value)
            .map_err(|SendError(_)| Error::ChannelDisconnected)
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// # Channel is disconnected
    ///
    /// This should only happen, if another thread has shut down. Within the
    /// scope of this application, this means that the overall system either
    /// already is shutting down, or should be going into shutdown.
    #[error("Channel disconnected")]
    ChannelDisconnected,

    #[error(transparent)]
    Other { err: anyhow::Error },
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other { err }
    }
}

#[derive(Debug)]
pub struct ThreadHandle {
    inner: Option<JoinHandle<anyhow::Result<()>>>,
}

impl ThreadHandle {
    pub fn new(handle: JoinHandle<anyhow::Result<()>>) -> Self {
        Self {
            inner: Some(handle),
        }
    }

    pub fn join(&mut self) -> anyhow::Result<()> {
        if self.inner.is_none() {
            panic!("You must not join an actor that has already been joined.");
        }

        if let Some(handle) = self.inner.take() {
            match handle.join() {
                Ok(result) => {
                    result?;
                }
                Err(payload) => {
                    panic::resume_unwind(payload);
                }
            }
        }

        Ok(())
    }
}
