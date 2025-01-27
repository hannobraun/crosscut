use std::collections::BTreeSet;

use crate::language::EditorInputEvent;

pub struct TerminalEditorInput {}

impl TerminalEditorInput {
    pub fn new() -> Self {
        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("nop");
        commands.insert("reset");

        Self {}
    }

    pub fn on_input(&mut self, event: TerminalInputEvent) {
        let event = match event {
            TerminalInputEvent::Character { ch } => {
                Some(EditorInputEvent::Character { ch })
            }
            TerminalInputEvent::Backspace => {
                Some(EditorInputEvent::RemoveCharacterLeft)
            }
            TerminalInputEvent::Enter => None,
            TerminalInputEvent::Left => Some(EditorInputEvent::MoveCursorLeft),
            TerminalInputEvent::Right => {
                Some(EditorInputEvent::MoveCursorRight)
            }
            TerminalInputEvent::Escape => None,
        };

        if let Some(event) = event {
            if let EditorInputEvent::Character { ch } = event {
                dbg!(ch);
            } else {
                dbg!(event);
            }
        }
    }
}

#[derive(Debug)]
pub enum TerminalInputEvent {
    Character { ch: char },

    Backspace,
    Enter,
    Left,
    Right,
    Escape,
}
