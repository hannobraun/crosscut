use std::{io::stdin, thread};

use crate::actor::{Sender, ThreadHandle};

pub fn start(lines: Sender<String>) -> ThreadHandle {
    let handle = thread::spawn(move || loop {
        let mut line = String::new();
        stdin().read_line(&mut line)?;
        lines.send(line)?;
    });

    ThreadHandle::new(handle)
}
