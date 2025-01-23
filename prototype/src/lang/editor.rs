use std::{
    collections::{BTreeSet, VecDeque},
    iter,
};

use crate::lang::{
    code::{Body, Code, Fragment, FragmentKind},
    compiler::compile_and_replace,
    host::Host,
    interpreter::Interpreter,
};

use super::code::Location;

/// # Platform-independent and I/O-less editor core
///
/// ## Implementation Note
///
/// For being platform-independent, the input that this API processes is a bit
/// too specific, dealing with the identity of specific keys.
///
/// For now, this is fine. But eventually, I think we should extract a
/// translation layer. This translation layer can still be platform-independent,
/// but can translate from specific key presses to more higher-level concepts,
/// like "go to parent" or "leave current context", instead of "up" and
/// "escape".
#[derive(Debug)]
pub struct Editor {
    editing: Location,
    mode: EditorMode,
    input: EditorInputState,
    error: Option<EditorError>,
    commands: BTreeSet<&'static str>,
}

impl Editor {
    pub fn new(code: &mut Code) -> Self {
        let editing = code.append_to(
            &code.find_innermost_fragment_with_valid_body(),
            Fragment {
                kind: FragmentKind::Empty,
                body: Body::default(),
            },
        );

        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("nop");
        commands.insert("reset");

        Self {
            editing,
            mode: EditorMode::Edit,
            input: EditorInputState::new(String::new()),
            error: None,
            commands,
        }
    }

    pub fn editing(&self) -> &Location {
        &self.editing
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn input(&self) -> &EditorInputState {
        &self.input
    }

    pub fn error(&self) -> Option<&EditorError> {
        self.error.as_ref()
    }

    pub fn on_input(
        &mut self,
        event: InputEvent,
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
                    self.process_command(code, interpreter);
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
                    if value.is_whitespace() {
                        self.editing = code.append_to(
                            &code.find_innermost_fragment_with_valid_body(),
                            Fragment {
                                kind: FragmentKind::Empty,
                                body: Body::default(),
                            },
                        );

                        self.input.clear();
                    } else {
                        self.input.insert(value);
                        self.process_code(code, interpreter, host);
                    }
                }
                InputEvent::Backspace => {
                    self.input.remove_left();
                    self.process_code(code, interpreter, host);
                }
                InputEvent::Enter => {}
                InputEvent::Left => {
                    self.input.move_cursor_left();
                }
                InputEvent::Right => {
                    self.input.move_cursor_right();
                }
                InputEvent::Escape => {
                    self.mode = EditorMode::Command {
                        input: EditorInputState::new(String::new()),
                    };
                }
            },
        }
    }

    fn process_code(
        &mut self,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        self.editing =
            compile_and_replace(&self.input.buffer, &self.editing, host, code);

        if interpreter.state(code).is_running() {
            interpreter.update(code);
        } else {
            interpreter.reset(code);
        }
    }

    pub fn process_command(
        &mut self,
        code: &mut Code,
        interpreter: &mut Interpreter,
    ) {
        // This is an ugly hack, but it's temporary, as I transition to making
        // "command mode" not be a thing here.
        let EditorMode::Command { ref mut input } = &mut self.mode else {
            unreachable!(
                "This method is never called, unless we're in command mode."
            );
        };
        let command = &input.buffer;

        self.error = None;

        let mut candidates = self
            .commands
            .iter()
            .filter(|c| c.starts_with(command))
            .collect::<VecDeque<_>>();

        let Some(&candidate) = candidates.pop_front() else {
            self.error = Some(EditorError::UnknownCommand {
                command: command.clone(),
            });
            return;
        };
        if !candidates.is_empty() {
            let candidates = iter::once(candidate)
                .chain(candidates.into_iter().copied())
                .collect();

            self.error = Some(EditorError::AmbiguousCommand {
                command: command.clone(),
                candidates,
            });

            return;
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

        self.on_command(command, code, interpreter);
    }

    pub fn on_command(
        &mut self,
        command: Command,
        code: &mut Code,
        interpreter: &mut Interpreter,
    ) {
        match command {
            Command::Clear => {
                *code = Code::default();
                *self = Self::new(code);
                interpreter.reset(code);
            }
            Command::Nop => {
                // This command does nothing. It exists to give tests something
                // to execute, if they don't want to actually do something
                // except test command interaction itself.
            }
            Command::Reset => {
                interpreter.reset(code);
            }
        }
    }
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

#[derive(Debug, Eq, PartialEq)]
pub struct EditorInputState {
    pub buffer: String,
    pub cursor: usize,
}

impl EditorInputState {
    pub fn new(buffer: String) -> Self {
        let cursor = buffer.chars().count();
        Self { buffer, cursor }
    }

    pub fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.move_cursor_right();
    }

    pub fn remove_left(&mut self) {
        if let Some(cursor) = self.cursor.checked_sub(1) {
            self.buffer.remove(cursor);
            self.move_cursor_left();
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor =
            usize::min(self.cursor.saturating_add(1), self.buffer.len());
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}

#[derive(Debug)]
pub enum InputEvent {
    Char { value: char },

    Backspace,
    Enter,
    Left,
    Right,
    Escape,
}

pub enum Command {
    Clear,
    Nop,
    Reset,
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
