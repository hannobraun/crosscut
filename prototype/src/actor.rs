use std::{
    panic,
    sync::mpsc::{self, RecvError, SendError, TryRecvError},
    thread::{self, JoinHandle},
};

pub struct Actor<I> {
    pub sender: Sender<I>,
    pub handle: ThreadHandle,
}

impl<I> Actor<I> {
    pub fn spawn<F>(
        sender: Sender<I>,
        receiver: Receiver<I>,
        mut f: F,
    ) -> Actor<I>
    where
        I: Send + 'static,
        F: FnMut(I) -> Result<(), Error> + Send + 'static,
    {
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

    (Sender { inner: sender }, Receiver { inner: receiver })
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

pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, Error> {
        self.inner
            .recv()
            .map_err(|RecvError| Error::ChannelDisconnected)
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.inner.try_recv()
    }
}

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
    inner: JoinHandle<anyhow::Result<()>>,
}

impl ThreadHandle {
    pub fn new(handle: JoinHandle<anyhow::Result<()>>) -> Self {
        Self { inner: handle }
    }

    pub fn join(self) -> anyhow::Result<()> {
        match self.inner.join() {
            Ok(result) => result,
            Err(payload) => {
                panic::resume_unwind(payload);
            }
        }
    }
}
