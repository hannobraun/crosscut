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
    input: EditorInput,
    error: Option<EditorError>,
    commands: BTreeSet<&'static str>,
}

impl Editor {
    pub fn new(code: &mut Code) -> Self {
        let location = code.append_to(
            &code.find_innermost_fragment_with_valid_body(),
            Fragment {
                kind: FragmentKind::Empty,
                body: Body::default(),
            },
        );

        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("edit");
        commands.insert("reset");

        Self {
            editing: location,
            mode: EditorMode::Edit,
            input: EditorInput::new(String::new()),
            error: None,
            commands,
        }
    }

    pub fn editing(&self) -> Option<&Location> {
        Some(&self.editing)
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn input(&self) -> &EditorInput {
        &self.input
    }

    pub fn error(&self) -> Option<&EditorError> {
        self.error.as_ref()
    }

    pub fn process_input(
        &mut self,
        input: InputEvent,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        match input {
            InputEvent::Char { value } => {
                if value.is_whitespace() {
                    if let EditorMode::Edit = &mut self.mode {
                        self.editing = code.append_to(
                            &code.find_innermost_fragment_with_valid_body(),
                            Fragment {
                                kind: FragmentKind::Empty,
                                body: Body::default(),
                            },
                        );

                        self.input.clear();
                    }
                } else {
                    self.input.insert(value);

                    if let EditorMode::Edit = &mut self.mode {
                        Self::process_code(
                            &mut self.input,
                            &mut self.editing,
                            code,
                            interpreter,
                            host,
                        );
                    }
                }
            }
            InputEvent::Backspace => {
                self.input.remove_left();

                if let EditorMode::Edit = &mut self.mode {
                    Self::process_code(
                        &mut self.input,
                        &mut self.editing,
                        code,
                        interpreter,
                        host,
                    );
                }
            }
            InputEvent::Enter => match &self.mode {
                EditorMode::Command => {
                    self.process_command(code, interpreter);
                    self.input.clear();
                }
                EditorMode::Edit { .. } => {
                    self.mode = EditorMode::Command;
                    self.input.clear();
                }
            },
            InputEvent::Left => {
                self.input.move_cursor_left();
            }
            InputEvent::Right => {
                self.input.move_cursor_right();
            }
        }
    }

    fn process_code(
        input: &mut EditorInput,
        to_replace: &mut Location,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        *to_replace =
            compile_and_replace(&input.buffer, to_replace, host, code);

        if interpreter.state(code).is_running() {
            interpreter.update(code);
        } else {
            interpreter.reset(code);
        }
    }

    fn process_command(
        &mut self,
        code: &mut Code,
        interpreter: &mut Interpreter,
    ) {
        self.error = None;
        let command = &self.input.buffer;

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

        match candidate {
            "clear" => {
                *code = Code::default();
                interpreter.reset(code);
            }
            "edit" => {
                let location = code.append_to(
                    &code.find_innermost_fragment_with_valid_body(),
                    Fragment {
                        kind: FragmentKind::Empty,
                        body: Body::default(),
                    },
                );
                self.editing = location;
                self.mode = EditorMode::Edit;
            }
            "reset" => {
                interpreter.reset(code);
            }
            _ => {
                unreachable!("Ruled out that command is unknown, above.")
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Command,
    Edit,
}

#[derive(Debug)]
pub struct EditorInput {
    pub buffer: String,
    pub cursor: usize,
}
impl EditorInput {
    fn new(buffer: String) -> Self {
        let cursor = buffer.chars().count();
        Self { buffer, cursor }
    }

    fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.move_cursor_right();
    }

    fn remove_left(&mut self) {
        if let Some(cursor) = self.cursor.checked_sub(1) {
            self.buffer.remove(cursor);
            self.move_cursor_left();
        }
    }

    fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_cursor_right(&mut self) {
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
