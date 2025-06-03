mod input;
mod output;
mod thread;

pub use self::{
    output::{Cursor, RawTerminalAdapter, TerminalOutputAdapter},
    thread::{ChannelDisconnected, Receiver, start},
};

#[cfg(test)]
pub use self::output::{DebugOutputAdapter, StringOutputAdapter};
