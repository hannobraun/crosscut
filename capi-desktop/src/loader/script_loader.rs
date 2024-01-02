use std::path::PathBuf;

use crossbeam_channel::{Receiver, SendError, Sender};

use super::load;

pub struct ScriptLoader {
    path: PathBuf,
    sender: Sender<anyhow::Result<String>>,
}

impl ScriptLoader {
    pub fn new(path: PathBuf) -> (Self, Receiver<anyhow::Result<String>>) {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        let self_ = Self { path, sender };
        (self_, receiver)
    }

    pub fn on_error(
        &self,
        err: impl Into<anyhow::Error>,
    ) -> Result<(), SendError<anyhow::Result<String>>> {
        self.sender.send(Err(err.into()))
    }

    pub fn trigger(&self) -> Result<(), SendError<anyhow::Result<String>>> {
        let code_or_err = load(&self.path);
        self.sender.send(code_or_err)
    }
}
