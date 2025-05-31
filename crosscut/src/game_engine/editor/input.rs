use crate::language::{
    editor::{EditorCommand, EditorInputBuffer, EditorInputEvent},
    language::Language,
};

#[derive(Debug)]
pub struct TerminalEditorInput {
    mode: EditorMode,
}

impl TerminalEditorInput {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Edit,
        }
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn on_input(
        &mut self,
        event: TerminalInput,
        language: &mut Language,
    ) -> anyhow::Result<()> {
        match &mut self.mode {
            EditorMode::Edit => match event {
                TerminalInput::Escape => {
                    self.mode = EditorMode::Command {
                        buffer: EditorInputBuffer::empty(),
                        cursor: 0,
                    };
                }
                event => {
                    if let Some(event) = event.into_editor_input_event() {
                        language.on_input(event);
                    }
                }
            },
            EditorMode::Command { buffer, cursor } => match event {
                TerminalInput::Enter => {
                    match buffer.contents().as_str() {
                        "clear" => {
                            language.on_command(EditorCommand::Clear)?;
                        }
                        "dump" => {
                            language.on_command(EditorCommand::Dump)?;
                        }
                        "reset" => {
                            language.on_command(EditorCommand::Reset)?;
                        }
                        _ => {
                            // This should result in an error message being
                            // displayed where the user can see it. For now, we
                            // just ignore it though.
                        }
                    }

                    self.mode = EditorMode::Edit;
                }
                TerminalInput::Escape => {
                    self.mode = EditorMode::Edit;
                }
                event => {
                    if let Some(event) = event.into_editor_input_event() {
                        buffer.update(event, cursor);
                    }
                }
            },
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Edit,
    Command {
        buffer: EditorInputBuffer,
        cursor: usize,
    },
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
pub enum TerminalInput {
    Character {
        ch: char,
    },

    Backspace {
        ctrl_pressed: bool,
    },
    Delete {
        ctrl_pressed: bool,
    },

    Left,
    Right,
    Up,
    Down,

    Enter,
    Escape,

    /// # An event that has no effect when processed
    ///
    /// If a thread shuts down, either because of an error, or because the
    /// application is supposed to shut down as a whole, that needs to propagate
    /// to the other threads.
    ///
    /// For some threads, this is easily achieved, because they block on reading
    /// from a channel from another thread, which will fail the moment that
    /// other thread shuts down. Other threads block on something else, and
    /// don't benefit from this mechanism.
    ///
    /// Those other threads need to instead _send_ to another thread from time
    /// to time, to learn about the shutdown. This is what this event is for.
    Heartbeat,
}

impl TerminalInput {
    fn into_editor_input_event(self) -> Option<EditorInputEvent> {
        match self {
            Self::Character { ch } if ch.is_whitespace() => {
                Some(EditorInputEvent::Submit)
            }
            Self::Character { ch } => Some(EditorInputEvent::Insert { ch }),

            Self::Backspace { ctrl_pressed } => {
                Some(EditorInputEvent::RemoveLeft {
                    whole_node: ctrl_pressed,
                })
            }
            Self::Delete { ctrl_pressed } => {
                Some(EditorInputEvent::RemoveRight {
                    whole_node: ctrl_pressed,
                })
            }

            Self::Left => Some(EditorInputEvent::MoveCursorLeft),
            Self::Right => Some(EditorInputEvent::MoveCursorRight),
            Self::Up => Some(EditorInputEvent::MoveCursorUp),
            Self::Down => Some(EditorInputEvent::MoveCursorDown),

            Self::Enter => Some(EditorInputEvent::MoveCursorDown),

            _ => None,
        }
    }
}
