use std::{ops::ControlFlow, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::game_engine::TerminalInputEvent;

pub fn read_editor_event()
-> anyhow::Result<ControlFlow<(), Option<TerminalInputEvent>>> {
    let timeout = Duration::from_millis(50);
    let event_ready = event::poll(timeout)?;

    if !event_ready {
        // We must send on the channel from time to time, to make sure we
        // learn if the other thread has shut down. Otherwise, this thread
        // will hang forever, blocking on input, preventing the application
        // from shutting down.
        return Ok(ControlFlow::Continue(None));
    }

    let event = event::read()?;

    let Event::Key(key_event) = event else {
        return Ok(ControlFlow::Continue(None));
    };

    let ctrl_pressed = key_event.modifiers.contains(KeyModifiers::CONTROL);

    let event = match key_event.code {
        KeyCode::Char('c') if ctrl_pressed => {
            // The terminal is in raw mode, so we have to handle CTRL+C
            // manually.
            //
            // Ending this thread is enough. It will drop its channel, which
            // will propagate the shutdown to all other threads.
            return Ok(ControlFlow::Break(()));
        }
        KeyCode::Char(ch) if ch.is_ascii() => {
            // Only ASCII characters are currently accepted. This limitation is
            // tracked here:
            // https://github.com/hannobraun/crosscut/issues/70

            Some(TerminalInputEvent::Character { ch })
        }

        KeyCode::Backspace => {
            Some(TerminalInputEvent::Backspace { ctrl_pressed })
        }
        KeyCode::Enter => Some(TerminalInputEvent::Enter),
        KeyCode::Left => Some(TerminalInputEvent::Left),
        KeyCode::Right => Some(TerminalInputEvent::Right),
        KeyCode::Up => Some(TerminalInputEvent::Up),
        KeyCode::Down => Some(TerminalInputEvent::Down),
        KeyCode::Delete => Some(TerminalInputEvent::Delete { ctrl_pressed }),
        KeyCode::Esc => Some(TerminalInputEvent::Escape),
        _ => None,
    };

    Ok(ControlFlow::Continue(event))
}
