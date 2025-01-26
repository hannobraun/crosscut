use std::{
    io,
    ops::ControlFlow,
    panic,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{select, SendError, TryRecvError};

use crate::game_engine::{GameInput, GameOutput};

pub fn start() -> anyhow::Result<Threads> {
    let (game_input_tx, game_input_rx) = channel();
    let (game_output_tx, game_output_rx) = channel();

    let game_engine = spawn(move || loop {
        let event = select! {
            recv(game_input_rx.inner) -> result => {
                result.map(|input| GameEngineEvent::GameInput { input })
            }
        };
        let Ok(event) = event else {
            return Err(ChannelDisconnected.into());
        };

        let GameEngineEvent::GameInput { input } = event;
        dbg!(input);

        game_output_tx.send(GameOutput::SubmitColor { color: [1.; 4] })?;
    });

    Ok(Threads {
        handles: [game_engine],
        game_input: game_input_tx,
        game_output: game_output_rx,
    })
}

pub struct Threads {
    pub handles: [ThreadHandle; 1],
    pub game_input: Sender<GameInput>,
    pub game_output: Receiver<GameOutput>,
}

#[derive(Debug)]
pub struct ThreadHandle {
    inner: JoinHandle<anyhow::Result<()>>,
}

impl ThreadHandle {
    fn new(handle: JoinHandle<anyhow::Result<()>>) -> Self {
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
    pub fn try_recv(&self) -> Result<Option<T>, ChannelDisconnected> {
        match self.inner.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ChannelDisconnected),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
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

fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = crossbeam_channel::unbounded();

    (Sender { inner: sender }, Receiver { inner: receiver })
}

fn spawn<F>(mut f: F) -> ThreadHandle
where
    F: FnMut() -> Result<ControlFlow<()>, Error> + Send + 'static,
{
    let handle = thread::spawn(move || {
        loop {
            match f() {
                Ok(ControlFlow::Continue(())) => {}
                Ok(ControlFlow::Break(())) => {
                    break;
                }
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

#[derive(Debug)]
enum GameEngineEvent {
    GameInput { input: GameInput },
}
