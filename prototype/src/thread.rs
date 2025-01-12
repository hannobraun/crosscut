use std::{
    io, panic,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{RecvError, SendError, TryRecvError};

pub fn spawn<F>(mut f: F) -> ThreadHandle
where
    F: FnMut() -> Result<(), Error> + Send + 'static,
{
    let handle = thread::spawn(move || {
        loop {
            match f() {
                Ok(()) => {}
                Err(Error::ChannelDisconnected { .. }) => {
                    break;
                }
                Err(Error::Other { err }) => {
                    return Err(err);
                }
            }
        }

        Ok(())
    });

    ThreadHandle::new(handle)
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = crossbeam_channel::unbounded();

    (Sender { inner: sender }, Receiver { inner: receiver })
}

pub struct Sender<T> {
    inner: crossbeam_channel::Sender<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), ChannelDisconnected> {
        self.inner
            .send(value)
            .map_err(|SendError(_)| ChannelDisconnected)
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
    inner: crossbeam_channel::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, ChannelDisconnected> {
        self.inner.recv().map_err(|RecvError| ChannelDisconnected)
    }

    pub fn try_recv(&self) -> Result<Option<T>, ChannelDisconnected> {
        match self.inner.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ChannelDisconnected),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// # Channel is disconnected
    ///
    /// This should only happen, if another thread has shut down. Within the
    /// scope of this application, this means that the overall system either
    /// already is shutting down, or should be going into shutdown.
    #[error(transparent)]
    ChannelDisconnected {
        #[from]
        err: ChannelDisconnected,
    },

    #[error(transparent)]
    Other {
        #[from]
        err: anyhow::Error,
    },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Other { err: err.into() }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Channel disconnected")]
pub struct ChannelDisconnected;

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
