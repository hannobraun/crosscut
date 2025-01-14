use std::{ops::ControlFlow, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::{
    editor::InputEvent,
    thread::{self, Sender, ThreadHandle},
};

pub fn start(editor_input: Sender<Option<InputEvent>>) -> ThreadHandle {
    thread::spawn(move || {
        let timeout = Duration::from_millis(50);
        let event_ready = event::poll(timeout)?;

        if !event_ready {
            // We must send on the channel from time to time, to make sure we
            // learn if the other thread has shut down. Otherwise, this thread
            // will hang forever, blocking on input, preventing the application
            // from shutting down.
            editor_input.send(None)?;
            return Ok(ControlFlow::Continue(()));
        }

        let event = event::read()?;

        let Event::Key(key_event) = event else {
            return Ok(ControlFlow::Continue(()));
        };

        match key_event.code {
            KeyCode::Char('c')
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                // Ending this thread is enough. It will drop its channel, which
                // will propagate the shutdown to all other threads.
                return Ok(ControlFlow::Break(()));
            }
            KeyCode::Char(ch) => {
                editor_input.send(Some(InputEvent::Char { value: ch }))?;
            }

            KeyCode::Backspace => {
                editor_input.send(Some(InputEvent::Backspace))?;
            }
            KeyCode::Enter => {
                editor_input.send(Some(InputEvent::Enter))?;
            }
            _ => {}
        }

        Ok(ControlFlow::Continue(()))
    })
}
