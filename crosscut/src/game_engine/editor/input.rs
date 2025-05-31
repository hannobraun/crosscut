use crate::language::{
    editor::{EditorCommand, EditorInput, EditorInputBuffer},
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
        input: TerminalInput,
        _: &mut Language,
    ) -> anyhow::Result<Option<EditorInputOrCommand>> {
        match &mut self.mode {
            EditorMode::Edit => match input {
                TerminalInput::Escape => {
                    self.mode = EditorMode::Command {
                        buffer: EditorInputBuffer::empty(),
                        cursor: 0,
                    };
                }
                event => {
                    return Ok(event
                        .into_editor_input_event()
                        .map(|input| EditorInputOrCommand::Input { input }));
                }
            },
            EditorMode::Command { buffer, cursor } => match input {
                TerminalInput::Enter => {
                    let command = match buffer.contents() {
                        "clear" => Some(EditorCommand::Clear),
                        "dump" => Some(EditorCommand::Dump),
                        "reset" => Some(EditorCommand::Reset),
                        _ => {
                            // This should result in an error message being
                            // displayed where the user can see it. For now, we
                            // just ignore it though.

                            None
                        }
                    };

                    self.mode = EditorMode::Edit;

                    return Ok(command.map(|command| {
                        EditorInputOrCommand::Command { command }
                    }));
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

        Ok(None)
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
    fn into_editor_input_event(self) -> Option<EditorInput> {
        match self {
            Self::Character { ch } if ch.is_whitespace() => {
                Some(EditorInput::Submit)
            }
            Self::Character { ch } => Some(EditorInput::Insert { ch }),

            Self::Backspace { ctrl_pressed } => Some(EditorInput::RemoveLeft {
                whole_node: ctrl_pressed,
            }),
            Self::Delete { ctrl_pressed } => Some(EditorInput::RemoveRight {
                whole_node: ctrl_pressed,
            }),

            Self::Left => Some(EditorInput::MoveCursorLeft),
            Self::Right => Some(EditorInput::MoveCursorRight),
            Self::Up => Some(EditorInput::MoveCursorUp),
            Self::Down => Some(EditorInput::MoveCursorDown),

            Self::Enter => Some(EditorInput::MoveCursorDown),

            _ => None,
        }
    }
}

pub enum EditorInputOrCommand {
    Input { input: EditorInput },
    Command { command: EditorCommand },
}
