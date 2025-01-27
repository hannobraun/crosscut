use std::collections::BTreeSet;

use crate::language::{
    code::Codebase,
    editor::{Editor, EditorInputEvent},
};

pub struct TerminalEditorInput {
    mode: EditorMode,
}

impl TerminalEditorInput {
    pub fn new() -> Self {
        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("nop");
        commands.insert("reset");

        Self {
            mode: EditorMode::Edit,
        }
    }

    pub fn on_input(
        &mut self,
        event: TerminalInputEvent,
        editor: &mut Editor,
        codebase: &mut Codebase,
    ) {
        match self.mode {
            EditorMode::Edit => {
                let event = match event {
                    TerminalInputEvent::Character { ch } => {
                        Some(EditorInputEvent::Character { ch })
                    }
                    TerminalInputEvent::Backspace => {
                        Some(EditorInputEvent::RemoveCharacterLeft)
                    }
                    TerminalInputEvent::Enter => None,
                    TerminalInputEvent::Left => {
                        Some(EditorInputEvent::MoveCursorLeft)
                    }
                    TerminalInputEvent::Right => {
                        Some(EditorInputEvent::MoveCursorRight)
                    }
                    TerminalInputEvent::Escape => None,
                };

                if let Some(event) = event {
                    editor.on_input(event, codebase)
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Edit,
}

#[derive(Debug)]
pub enum TerminalInputEvent {
    Character { ch: char },

    Backspace,
    Left,
    Right,

    Enter,
    Escape,
}
