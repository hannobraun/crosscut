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
pub struct Editor {
    mode: EditorMode,
    input: EditorInput,
    error: Option<EditorError>,
    commands: BTreeSet<&'static str>,
}

impl Editor {
    pub fn new() -> Self {
        // All of the trie crates I could find where overly complex, unsuitable
        // for my use case, or dubious in other ways. Let's just do this by
        // hand.

        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("edit");
        commands.insert("reset");

        Self {
            mode: EditorMode::Command,
            input: EditorInput::new(String::new()),
            error: None,
            commands,
        }
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
                    if let EditorMode::Edit = self.mode {
                        let to_replace = code.append_to(
                            &code.find_innermost_fragment_with_valid_body(),
                            Fragment {
                                kind: FragmentKind::Empty,
                                body: Body::default(),
                            },
                        );

                        self.process_code(to_replace, code, interpreter, host);
                    }
                } else {
                    self.input.insert(value);
                }
            }

            InputEvent::Backspace => {
                self.input.remove_left();
            }
            InputEvent::Enter => match self.mode {
                EditorMode::Command => {
                    self.process_command(code, interpreter);
                    self.input.clear();
                }
                EditorMode::Edit => {
                    let to_replace = code.append_to(
                        &code.find_innermost_fragment_with_valid_body(),
                        Fragment {
                            kind: FragmentKind::Empty,
                            body: Body::default(),
                        },
                    );
                    self.process_code(to_replace, code, interpreter, host);
                    self.mode = EditorMode::Command;
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
        &mut self,
        to_replace: Location,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        compile_and_replace(&self.input.buffer, &to_replace, host, code);

        self.input.clear();

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

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Command,
    Edit,
}

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

pub enum EditorError {
    AmbiguousCommand {
        command: String,
        candidates: Vec<&'static str>,
    },
    UnknownCommand {
        command: String,
    },
}
