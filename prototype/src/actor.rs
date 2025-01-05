use std::{
    sync::mpsc::{self, RecvError, SendError},
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

            if let Err(ChannelError::Disconnected) = self.sender.send(input) {
                break;
            }
        });

        self.handle
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

pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        self.inner.recv()
    }

    pub fn into_inner(self) -> mpsc::Receiver<T> {
        self.inner
    }
}

pub enum ChannelError {
    Disconnected,
}
