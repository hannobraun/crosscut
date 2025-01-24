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

pub struct EditorInput {
    mode: EditorMode,
    error: Option<EditorError>,
    commands: BTreeSet<&'static str>,
}

impl EditorInput {
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
        event: InputEvent,
        editor: &mut Editor,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        match &mut self.mode {
            EditorMode::Command { input } => match event {
                InputEvent::Char { value } => {
                    input.insert(value);
                }
                InputEvent::Backspace => {
                    input.remove_left();
                }
                InputEvent::Enter => {
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
                InputEvent::Left => {
                    input.move_cursor_left();
                }
                InputEvent::Right => {
                    input.move_cursor_right();
                }
                InputEvent::Escape => {
                    self.mode = EditorMode::Edit;
                }
            },
            EditorMode::Edit => match event {
                InputEvent::Char { value } => {
                    editor.on_input(
                        InputEvent::Char { value },
                        code,
                        interpreter,
                        host,
                    );
                }
                InputEvent::Backspace => {
                    editor.on_input(
                        InputEvent::Backspace,
                        code,
                        interpreter,
                        host,
                    );
                }
                InputEvent::Enter => {
                    editor.on_input(InputEvent::Enter, code, interpreter, host);
                }
                InputEvent::Left => {
                    editor.on_input(InputEvent::Left, code, interpreter, host);
                }
                InputEvent::Right => {
                    editor.on_input(InputEvent::Right, code, interpreter, host);
                }
                InputEvent::Escape => {
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
