use std::thread;

use crossterm::event::{self, Event, KeyCode};

use crate::thread::{Sender, ThreadHandle};

pub fn start(editor_input: Sender<String>) -> ThreadHandle {
    let mut line = String::new();

    let handle = thread::spawn(move || loop {
        let event = event::read()?;

        let Event::Key(key_event) = event else {
            continue;
        };

        match key_event.code {
            KeyCode::Char(ch) => {
                line.push(ch);
            }
            KeyCode::Enter => {
                editor_input.send(line.clone())?;
                line.clear();
            }
            _ => {
                continue;
            }
        }
    });

    ThreadHandle::new(handle)
}
