use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};

use crate::thread::{self, Sender, ThreadHandle};

pub fn start(editor_input: Sender<Option<String>>) -> ThreadHandle {
    let mut line = String::new();

    thread::spawn(move || {
        let timeout = Duration::from_millis(500);
        let event_ready = event::poll(timeout)?;

        if !event_ready {
            // We must send on the channel from time to time, to make sure we
            // learn if the other thread has shut down. Otherwise, this thread
            // will hang forever, blocking on input, preventing the application
            // from shutting down.
            editor_input.send(None)?;
            return Ok(());
        }

        let event = event::read()?;

        let Event::Key(key_event) = event else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Char(ch) => {
                line.push(ch);
            }
            KeyCode::Enter => {
                editor_input.send(Some(line.clone()))?;
                line.clear();
            }
            _ => {}
        }

        Ok(())
    })
}
