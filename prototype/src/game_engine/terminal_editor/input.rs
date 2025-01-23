use std::{
    collections::{BTreeSet, VecDeque},
    iter,
};

use crate::lang::{
    code::Code,
    editor::{Command, Editor, EditorError, EditorInputState, InputEvent},
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
                    self.error = process_command(
                        input,
                        &self.commands,
                        editor,
                        code,
                        interpreter,
                    );
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
                InputEvent::Escape => {
                    self.mode = EditorMode::Command {
                        input: EditorInputState::new(String::new()),
                    };
                }
                event => {
                    editor.on_input(event, code, interpreter, host);
                }
            },
        }
    }
}

fn process_command(
    input: &mut EditorInputState,
    commands: &BTreeSet<&'static str>,
    editor: &mut Editor,
    code: &mut Code,
    interpreter: &mut Interpreter,
) -> Option<EditorError> {
    let command = &input.buffer;

    let mut candidates = commands
        .iter()
        .filter(|c| c.starts_with(command))
        .collect::<VecDeque<_>>();

    let Some(&candidate) = candidates.pop_front() else {
        return Some(EditorError::UnknownCommand {
            command: command.clone(),
        });
    };
    if !candidates.is_empty() {
        let candidates = iter::once(candidate)
            .chain(candidates.into_iter().copied())
            .collect();

        return Some(EditorError::AmbiguousCommand {
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

    editor.on_command(command, code, interpreter);

    None
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
