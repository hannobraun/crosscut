use std::io::stdin;

use crate::actor::{Actor, Sender, ThreadHandle};

pub fn start(lines: Sender<String>) -> ThreadHandle {
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
