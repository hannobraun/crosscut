use std::{ops::ControlFlow, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::game_engine::TerminalInputEvent;

pub fn read_editor_event(
) -> anyhow::Result<ControlFlow<(), Option<TerminalInputEvent>>> {
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

    let event = match key_event.code {
        KeyCode::Char('c')
            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            // The terminal is in raw mode, so we have to handle CTRL+C
            // manually.
            //
            // Ending this thread is enough. It will drop its channel, which
            // will propagate the shutdown to all other threads.
            return Ok(ControlFlow::Break(()));
        }
        KeyCode::Char(ch) if ch.is_ascii() => {
            // We have code that needs to keep track of the cursor. That
            // code won't work with most Unicode characters, and I don't
            // know how to fix that. It's a complicated topic.
            //
            // Long-term, the terminal-based interface can only be a
            // placeholder anyway. So I think restricting input to ASCII
            // characters is a reasonable compromise.

            Some(TerminalInputEvent::Char { value: ch })
        }

        KeyCode::Backspace => Some(TerminalInputEvent::Backspace),
        KeyCode::Enter => Some(TerminalInputEvent::Enter),
        KeyCode::Left => Some(TerminalInputEvent::Left),
        KeyCode::Right => Some(TerminalInputEvent::Right),
        KeyCode::Esc => Some(TerminalInputEvent::Escape),
        _ => None,
    };

    Ok(ControlFlow::Continue(event))
}
