use std::time::Instant;

pub trait Game {}

pub struct PureCrosscutGame;

impl Game for PureCrosscutGame {}

#[derive(Debug)]
pub enum State {
    Running,
    EndOfFrame,
    WaitUntil { instant: Instant },
}
