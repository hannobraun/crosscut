use std::{
    collections::{BTreeSet, VecDeque},
    iter,
};

use crate::lang::{
    code::Code,
    editor::{Command, Editor, EditorInputState, InputEvent},
    host::Host,
    interpreter::Interpreter,
};

pub struct TerminalEditorInput {
    mode: EditorMode,
    error: Option<EditorError>,
    commands: BTreeSet<&'static str>,
}

impl TerminalEditorInput {
    pub fn new() -> Self {
        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("nop");
        commands.insert("reset");

        Self {
            mode: EditorMode::Edit,
            error: None,
            commands,
        }
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn error(&self) -> Option<&EditorError> {
        self.error.as_ref()
    }

    pub fn on_input(
        &mut self,
        event: TerminalInputEvent,
        editor: &mut Editor,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        match &mut self.mode {
            EditorMode::Command { input } => match event {
                TerminalInputEvent::Char { value } => {
                    input.insert(value);
                }
                TerminalInputEvent::Backspace => {
                    input.remove_left();
                }
                TerminalInputEvent::Enter => {
                    match parse_command(input, &self.commands) {
                        Ok(command) => {
                            editor.on_command(command, code, interpreter);
                        }
                        Err(err) => {
                            self.error = Some(err);
                        }
                    }

                    self.mode = EditorMode::Edit;
                }
                TerminalInputEvent::Left => {
                    input.move_cursor_left();
                }
                TerminalInputEvent::Right => {
                    input.move_cursor_right();
                }
                TerminalInputEvent::Escape => {
                    self.mode = EditorMode::Edit;
                }
            },
            EditorMode::Edit => match event {
                TerminalInputEvent::Char { value } => {
                    editor.on_input(
                        InputEvent::Char { value },
                        code,
                        interpreter,
                        host,
                    );
                }
                TerminalInputEvent::Backspace => {
                    editor.on_input(
                        InputEvent::Backspace,
                        code,
                        interpreter,
                        host,
                    );
                }
                TerminalInputEvent::Enter => {
                    editor.on_input(InputEvent::Enter, code, interpreter, host);
                }
                TerminalInputEvent::Left => {
                    editor.on_input(InputEvent::Left, code, interpreter, host);
                }
                TerminalInputEvent::Right => {
                    editor.on_input(InputEvent::Right, code, interpreter, host);
                }
                TerminalInputEvent::Escape => {
                    self.mode = EditorMode::Command {
                        input: EditorInputState::new(String::new()),
                    };
                }
            },
        }
    }
}

fn parse_command(
    input: &mut EditorInputState,
    commands: &BTreeSet<&'static str>,
) -> Result<Command, EditorError> {
    let command = &input.buffer;

    let mut candidates = commands
        .iter()
        .filter(|c| c.starts_with(command))
        .collect::<VecDeque<_>>();

    let Some(&candidate) = candidates.pop_front() else {
        return Err(EditorError::UnknownCommand {
            command: command.clone(),
        });
    };
    if !candidates.is_empty() {
        let candidates = iter::once(candidate)
            .chain(candidates.into_iter().copied())
            .collect();

        return Err(EditorError::AmbiguousCommand {
            command: command.clone(),
            candidates,
        });
    }

    input.clear();

    let command = match candidate {
        "clear" => Command::Clear,
        "nop" => Command::Nop,
        "reset" => Command::Reset,
        _ => {
            unreachable!("Ruled out that command is unknown, above.")
        }
    };

    Ok(command)
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Command { input: EditorInputState },
    Edit,
}

impl EditorMode {
    pub fn is_edit(&self) -> bool {
        matches!(self, Self::Edit)
    }
}

#[derive(Debug)]
pub enum EditorError {
    AmbiguousCommand {
        command: String,
        candidates: Vec<&'static str>,
    },
    UnknownCommand {
        command: String,
    },
}

#[derive(Debug)]
pub enum TerminalInputEvent {
    Char { value: char },

    Backspace,
    Enter,
    Left,
    Right,
    Escape,
}
