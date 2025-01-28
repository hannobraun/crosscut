use std::collections::BTreeSet;

use crate::language::{
    editor::{EditorCommand, EditorInput, EditorInputEvent},
    instance::Language,
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

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn on_input(
        &mut self,
        event: TerminalInputEvent,
        language: &mut Language,
    ) {
        match self.mode {
            EditorMode::Edit => {
                let event = match event {
                    TerminalInputEvent::Enter => None,
                    TerminalInputEvent::Escape => {
                        self.mode = EditorMode::Command {
                            input: EditorInput::empty(),
                        };
                        None
                    }
                    event => event.into_editor_input_event(),
                };

                if let Some(event) = event {
                    language.on_input(event);
                }
            }
            EditorMode::Command { .. } => match event {
                TerminalInputEvent::Enter => {
                    language.editor.on_command(
                        EditorCommand::Clear,
                        &mut language.codebase,
                    );
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
    Command { input: EditorInput },
}

impl EditorMode {
    #[cfg(test)]
    pub fn is_command_mode(&self) -> bool {
        matches!(self, Self::Command { .. })
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
    Delete,

    Left,
    Right,

    Enter,
    Escape,
}

impl TerminalInputEvent {
    fn into_editor_input_event(self) -> Option<EditorInputEvent> {
        match self {
            Self::Character { ch } => Some(EditorInputEvent::Insert { ch }),

            Self::Backspace => Some(EditorInputEvent::RemoveLeft),
            Self::Delete => Some(EditorInputEvent::RemoveRight),

            Self::Left => Some(EditorInputEvent::MoveCursorLeft),
            Self::Right => Some(EditorInputEvent::MoveCursorRight),

            _ => None,
        }
    }
}
