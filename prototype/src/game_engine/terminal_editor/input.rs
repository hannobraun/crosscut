use std::collections::BTreeSet;

use crate::language::{
    code::Codebase,
    editor::{Editor, EditorCommand, EditorInputEvent},
};

#[derive(Debug)]
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

    #[cfg(test)]
    pub fn mode(&self) -> &EditorMode {
        &self.mode
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
                    TerminalInputEvent::Left => {
                        Some(EditorInputEvent::MoveCursorLeft)
                    }
                    TerminalInputEvent::Right => {
                        Some(EditorInputEvent::MoveCursorRight)
                    }

                    TerminalInputEvent::Enter => None,
                    TerminalInputEvent::Escape => {
                        self.mode = EditorMode::Command;
                        None
                    }
                };

                if let Some(event) = event {
                    editor.on_input(event, codebase)
                }
            }
            EditorMode::Command => match event {
                TerminalInputEvent::Enter => {
                    editor.on_command(EditorCommand::Clear, codebase);
                    self.mode = EditorMode::Edit;
                }
                TerminalInputEvent::Escape => {
                    self.mode = EditorMode::Edit;
                }
                _ => {}
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Edit,
    Command,
}

impl EditorMode {
    #[cfg(test)]
    pub fn is_command_mode(&self) -> bool {
        matches!(self, Self::Command)
    }

    #[cfg(test)]
    pub fn is_edit_mode(&self) -> bool {
        matches!(self, Self::Edit)
    }
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
