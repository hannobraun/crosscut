use std::{
    panic,
    sync::mpsc::{self, SendError},
    thread::{self, JoinHandle},
};

pub struct Actor<I> {
    pub sender: Sender<I>,
    pub handle: ActorHandle,
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
                        break;
                    }
                }
            }

            Ok(())
        });

        Actor {
            sender,
            handle: ActorHandle {
                main: Some(handle),
                input: None,
            },
        }
    }

    pub fn provide_input<F>(mut self, mut f: F) -> ActorHandle
    where
        I: Send + 'static,
        F: FnMut() -> anyhow::Result<I> + Send + 'static,
    {
        let handle = thread::spawn(move || {
            loop {
                let input = f()?;

                if let Err(ChannelError::Disconnected) = self.sender.send(input)
                {
                    break;
                }
            }

            Ok(())
        });

        self.handle.input = Some(handle);
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
    /// # Channel is disconnected
    ///
    /// This should only happen, if another thread has shut down. Within the
    /// scope of this application, this means that the overall system either
    /// already is shutting down, or should be going into shutdown.
    Disconnected,
}

#[derive(Debug)]
pub struct ActorHandle {
    main: Option<JoinHandle<anyhow::Result<()>>>,
    input: Option<JoinHandle<anyhow::Result<()>>>,
}

impl ActorHandle {
    pub fn join(&mut self) -> anyhow::Result<()> {
        if self.main.is_none() {
            panic!("You must not join an actor that has already been joined.");
        }

        for handle in
            [self.main.take(), self.input.take()].into_iter().flatten()
        {
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

impl Drop for ActorHandle {
    fn drop(&mut self) {
        eprintln!(
            "WARNING: Dropping actor handle without having joined it. This is \
            fine, if it happens because of some other error. But if it happens \
            as part of a normal shutdown, it should be considered a bug."
        );
    }
}
