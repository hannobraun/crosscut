use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};

use crate::{
    editor::EditorInput,
    thread::{self, Sender, ThreadHandle},
};

pub fn start(editor_input: Sender<Option<EditorInput>>) -> ThreadHandle {
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
                let input = EditorInput { line: line.clone() };
                editor_input.send(Some(input))?;
                line.clear();
            }
            _ => {}
        }

        Ok(())
    })
}
