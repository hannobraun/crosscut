use std::{
    backtrace::Backtrace,
    io,
    ops::ControlFlow,
    panic,
    thread::{self, JoinHandle},
};

use anyhow::anyhow;
use crossbeam_channel::{SendError, TryRecvError};

use crate::{
    game_engine::TerminalInput,
    terminal::{RawTerminalAdapter, input::read_terminal_input},
};

pub fn start() -> anyhow::Result<TerminalThread> {
    // Since one of the threads puts the terminal into raw mode while it's
    // running, the default panic handler won't work well. Let's register a hook
    // that extracts all information we need, so we can later print it here,
    // after all other threads have ended.
    panic::set_hook(Box::new(|info| {
        RawTerminalAdapter::on_shutdown();

        let message = panic_message::panic_info_message(info);
        let location = info.location();
        let backtrace = Backtrace::force_capture();

        let thread = thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>");

        let location = if let Some(location) = location {
            format!(" at {location}")
        } else {
            String::new()
        };

        let message = format!(
            "Thread `{thread_name}` panicked{location}:\n\
            {message}\n\
            \n\
            {backtrace}"
        );

        eprintln!("{message}");

        // NOTE(hannobraun): Without this, I'm seeing overlap between the stack
        // trace and the terminal prompt. Possibly some weird interaction with
        // the alternate screen that the terminal uses, but I don't know.
        for _ in 0..message.lines().count() / 2 {
            eprintln!();
        }
    }));

    let (input_tx, input_rx) = channel();

    let handle = spawn("terminal input", move || {
        loop {
            match read_terminal_input() {
                Ok(ControlFlow::Continue(input)) => {
                    input_tx.send(input)?;
                }
                Ok(ControlFlow::Break(())) => break Ok(()),
                Err(err) => break Err(Error::Other { err }),
            }
        }
    })?;

    Ok(TerminalThread {
        handle,
        input: input_rx,
    })
}

pub struct TerminalThread {
    pub handle: ThreadHandle,
    pub input: Receiver<TerminalInput>,
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
            Err(_) => {
                // The panic handler already prints to stderr. Nothing more to
                // do here.

                Err(anyhow!("Thread panicked."))
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

fn spawn<F>(name: &str, f: F) -> anyhow::Result<ThreadHandle>
where
    F: FnOnce() -> Result<(), Error> + Send + 'static,
{
    let handle =
        thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                match f() {
                    Ok(()) | Err(Error::ChannelDisconnected { .. }) => {}
                    Err(Error::Other { err }) => {
                        return Err(err);
                    }
                }

                Ok(())
            })?;

    Ok(ThreadHandle::new(handle))
}
