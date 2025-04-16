use std::{
    backtrace::Backtrace,
    collections::HashMap,
    io,
    ops::ControlFlow,
    panic,
    sync::{LazyLock, Mutex},
    thread::{self, JoinHandle, ThreadId},
};

use anyhow::anyhow;
use crossbeam_channel::{SendError, TryRecvError, select};

use crate::{
    game_engine::{GameEngine, GameInput, GameOutput, TerminalInputEvent},
    io::editor::input::read_editor_event,
};

static PANICS: LazyLock<Mutex<HashMap<ThreadId, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn start() -> anyhow::Result<Threads> {
    // Since one of the threads puts the terminal into raw mode while it's
    // running, the default panic handler won't work well. Let's register a hook
    // that extracts all information we need, so we can later print it here,
    // after all other threads have ended.
    panic::set_hook(Box::new(|info| {
        let message = panic_message::panic_info_message(info);
        let location = info.location();
        let backtrace = Backtrace::force_capture();

        let thread = thread::current();
        let thread_id = thread.id();
        let thread_name = thread.name().unwrap_or("<unnamed>");

        let Ok(mut panics) = PANICS.lock() else {
            // Lock is poisoned. Nothing we can do about that, I think.
            return;
        };

        let location = if let Some(location) = location {
            format!(" at {location}")
        } else {
            String::new()
        };
        let full_message = format!(
            "Thread `{thread_name}` panicked{location}:\n\
            {message}\n\
            \n\
            {backtrace}"
        );

        panics.insert(thread_id, full_message);
    }));

    // Need to specify the types of the channels explicitly, to work around this
    // bug in rust-analyzer:
    // https://github.com/rust-lang/rust-analyzer/issues/15984
    let (editor_input_tx, editor_input_rx) =
        channel::<Option<TerminalInputEvent>>();
    let (game_input_tx, game_input_rx) = channel::<GameInput>();
    let (game_output_tx, game_output_rx) = channel();

    let mut game_engine = GameEngine::with_editor_ui()?;

    let editor_input =
        spawn("editor input", move || match read_editor_event() {
            Ok(ControlFlow::Continue(event)) => {
                editor_input_tx.send(event)?;
                Ok(ControlFlow::Continue(()))
            }
            Ok(ControlFlow::Break(())) => Ok(ControlFlow::Break(())),
            Err(err) => Err(Error::Other { err }),
        })?;

    let game_engine = spawn("game engine", move || {
        let event = select! {
            recv(editor_input_rx.inner) -> result => {
                result.map(|maybe_event|
                    if let Some(event) = maybe_event {
                        GameEngineEvent::EditorInput { event }}
                    else {
                        GameEngineEvent::Heartbeat
                    }
                )
            }
            recv(game_input_rx.inner) -> result => {
                result.map(|input| GameEngineEvent::GameInput { input })
            }
        };
        let Ok(event) = event else {
            return Err(ChannelDisconnected.into());
        };

        match event {
            GameEngineEvent::EditorInput { event } => {
                game_engine.on_editor_input(event)?;

                for event in game_engine.game_output() {
                    game_output_tx.send(event)?;
                }
            }
            GameEngineEvent::GameInput {
                input: GameInput::RenderingFrame,
            } => {
                // If a new frame is being rendered on the other thread, then
                // the game engine can get ready to provide the next one.
                game_engine.on_frame()?;
            }
            GameEngineEvent::Heartbeat => {}
        }

        Ok(ControlFlow::Continue(()))
    })?;

    Ok(Threads {
        handles: [editor_input, game_engine],
        game_input: game_input_tx,
        game_output: game_output_rx,
    })
}

pub struct Threads {
    pub handles: [ThreadHandle; 2],
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
        let thread_id = self.inner.thread().id();

        match self.inner.join() {
            Ok(result) => result,
            Err(_) => {
                let Ok(panics) = PANICS.lock() else {
                    return Err(anyhow!(
                        "Could not acquire panic info, because lock is \
                        poisoned."
                    ));
                };

                let Some(message) = panics.get(&thread_id) else {
                    unreachable!(
                        "Thread panicked, but panic hook doesn't seem to have \
                        run."
                    );
                };

                Err(anyhow!("{message}"))
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

fn spawn<F>(name: &str, mut f: F) -> anyhow::Result<ThreadHandle>
where
    F: FnMut() -> Result<ControlFlow<()>, Error> + Send + 'static,
{
    let handle =
        thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
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
            })?;

    Ok(ThreadHandle::new(handle))
}

#[derive(Debug)]
enum GameEngineEvent {
    EditorInput {
        event: TerminalInputEvent,
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
