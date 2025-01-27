use std::collections::BTreeSet;

use crate::language::{Editor, EditorInputEvent};

pub struct TerminalEditorInput {}

impl TerminalEditorInput {
    pub fn new() -> Self {
        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("nop");
        commands.insert("reset");

        Self {}
    }

    pub fn on_input(&mut self, event: TerminalInputEvent, editor: &mut Editor) {
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
            editor.on_input(event)
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
