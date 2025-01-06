use std::io::stdin;

use crate::actor::{Actor, ActorHandle, Sender};

pub fn start(commands: Sender<String>) -> ActorHandle {
    Actor::spawn(move |command| {
        commands.send(command)?;

        Ok(())
    })
    .provide_input(|| {
        let mut command = String::new();
        stdin().read_line(&mut command)?;

        Ok(command)
    })
}
