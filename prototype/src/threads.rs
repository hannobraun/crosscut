use std::{
    io,
    ops::ControlFlow,
    panic,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{select, SendError, TryRecvError};

use crate::{
    game_engine::{GameEngine, GameInput, GameOutput},
    lang::editor,
};

pub struct GameEngineThread {
    pub handle: ThreadHandle,
    pub editor_input: Sender<Option<editor::InputEvent>>,
    pub game_input: Sender<GameInput>,
}

impl GameEngineThread {
    pub fn start(game_output_tx: Sender<GameOutput>) -> anyhow::Result<Self> {
        let mut game_engine = GameEngine::new()?;
        game_engine.render_editor()?;

        // Need to specify the types of the channels explicitly, to work around
        // this bug in rust-analyzer:
        // https://github.com/rust-lang/rust-analyzer/issues/15984
        let (editor_input_tx, editor_input_rx) =
            channel::<Option<editor::InputEvent>>();
        let (game_input_tx, game_input_rx) = channel::<GameInput>();

        let handle = spawn(move || {
            let event = select! {
                recv(editor_input_rx.inner()) -> result => {
                    result.map(|maybe_event|
                        if let Some(event) = maybe_event {
                            GameEngineEvent::EditorInput { event }}
                        else {
                            GameEngineEvent::Heartbeat
                        }
                    )
                }
                recv(game_input_rx.inner()) -> result => {
                    result.map(|input| GameEngineEvent::GameInput { input })
                }
            };
            let Ok(event) = event else {
                return Err(ChannelDisconnected.into());
            };

            match event {
                GameEngineEvent::EditorInput { event } => {
                    let mut game_events = Vec::new();
                    game_engine.on_editor_input(event, &mut game_events)?;

                    for event in game_events {
                        game_output_tx.send(event)?;
                    }
                }
                GameEngineEvent::GameInput {
                    input: GameInput::RenderingFrame,
                } => {
                    // This loop is coupled to the frame rate of the renderer.
                }
                GameEngineEvent::Heartbeat => {}
            }

            Ok(ControlFlow::Continue(()))
        });

        Ok(Self {
            handle,
            editor_input: editor_input_tx,
            game_input: game_input_tx,
        })
    }
}

#[derive(Debug)]
enum GameEngineEvent {
    EditorInput {
        event: editor::InputEvent,
    },

    GameInput {
        input: GameInput,
    },

    /// # An event that has no effect when processed
    ///
    /// If a thread shuts down, either because of an error, or because the
    /// application is supposed to shut down as a whole, that needs to propagate
    /// to the other threads.
    ///
    /// For some threads, this is easily achieved, because they block on reading
    /// from a channel from another thread, which will fail the moment that
    /// other thread shuts down. Other threads block on something else, and
    /// don't benefit from this mechanism.
    ///
    /// Those other threads need to instead _send_ to another thread from time
    /// to time, to learn about the shutdown. This is what this event is for.
    Heartbeat,
}

pub fn spawn<F>(mut f: F) -> ThreadHandle
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
    pub fn try_recv(&self) -> Result<Option<T>, ChannelDisconnected> {
        match self.inner.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(ChannelDisconnected),
        }
    }

    pub fn inner(&self) -> &crossbeam_channel::Receiver<T> {
        &self.inner
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
