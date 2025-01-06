use std::io::stdin;

use crate::actor::{Actor, ActorHandle, Sender};

pub fn start(lines: Sender<String>) -> ActorHandle {
    Actor::spawn(move |line| {
        lines.send(line)?;
        Ok(())
    })
    .provide_input(|| {
        let mut line = String::new();
        stdin().read_line(&mut line)?;
        Ok(line)
    })
}
