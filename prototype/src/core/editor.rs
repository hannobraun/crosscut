use std::{
    collections::{BTreeSet, VecDeque},
    fmt::Write,
};

use crate::core::{
    code::{Body, Code, Fragment, FragmentKind},
    compiler::compile_and_replace,
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

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
    input: Input,
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
            input: Input::new(String::new()),
            error: None,
            commands,
        }
    }

    pub fn prompt(&self) -> EditorPrompt {
        let error = self.error.as_ref();

        EditorPrompt {
            mode: &self.mode,
            input: &self.input,
            error,
        }
    }

    pub fn process_input(
        &mut self,
        input: InputEvent,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) -> anyhow::Result<()> {
        match input {
            InputEvent::Char { value } => {
                if value.is_whitespace() {
                    if let EditorMode::Edit = self.mode {
                        self.process_code(code, interpreter, host);
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
                    self.process_command(code, interpreter)?;
                    self.input.clear();
                }
                EditorMode::Edit => {
                    self.process_code(code, interpreter, host);
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

        Ok(())
    }

    fn process_code(
        &mut self,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        let to_replace = code.append_to(
            &code.find_innermost_fragment_with_valid_body(),
            Fragment {
                kind: FragmentKind::Empty,
                body: Body::default(),
            },
        );

        compile_and_replace(&self.input.buffer, &to_replace, host, code);

        self.input.clear();

        let is_running =
            matches!(interpreter.state(code), InterpreterState::Running);

        if !is_running {
            interpreter.reset(code);
        }
    }

    fn process_command(
        &mut self,
        code: &mut Code,
        interpreter: &mut Interpreter,
    ) -> anyhow::Result<()> {
        self.error = None;
        let command = &self.input.buffer;

        let mut matched_commands = self
            .commands
            .iter()
            .filter(|c| c.starts_with(command))
            .collect::<VecDeque<_>>();

        let Some(&matched_command) = matched_commands.pop_front() else {
            self.error = Some(EditorError::Other {
                message: format!("Unknown command: `{command}`"),
            });
            return Ok(());
        };
        if !matched_commands.is_empty() {
            let mut error = format!(
                "`{command}` could refer to multiple commands: \
                `{matched_command}`"
            );
            for matched_command in matched_commands {
                write!(error, ", `{matched_command}`")?;
            }

            self.error = Some(EditorError::Other { message: error });

            return Ok(());
        }

        match matched_command {
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

        Ok(())
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum EditorMode {
    Command,
    Edit,
}

pub struct Input {
    pub buffer: String,
    pub cursor: usize,
}
impl Input {
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

pub struct EditorPrompt<'r> {
    pub mode: &'r EditorMode,
    pub input: &'r Input,
    pub error: Option<&'r EditorError>,
}

pub enum EditorError {
    Other { message: String },
}
