use std::{
    collections::{BTreeSet, VecDeque},
    fmt::Write,
};

use crate::language::{
    code::Code,
    compiler::compile,
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

pub struct Editor {
    code: Code,
    mode: EditorMode,
    input: Input,
    error: Option<String>,
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
            code: Code::default(),
            mode: EditorMode::Command,
            input: Input::new(String::new()),
            error: None,
            commands,
        }
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn prompt(&self) -> EditorPrompt {
        EditorPrompt {
            mode: &self.mode,
            input: &self.input,
            error: self.error.as_ref(),
        }
    }

    pub fn process_input(
        &mut self,
        input: InputEvent,
        host: &Host,
        interpreter: &mut Interpreter,
    ) -> anyhow::Result<()> {
        match input {
            InputEvent::Char { value } => {
                if value.is_whitespace() {
                    if let EditorMode::Edit = self.mode {
                        self.process_code(host, interpreter);
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
                    self.process_command(interpreter)?;
                    self.input.clear();
                }
                EditorMode::Edit => {
                    self.process_code(host, interpreter);
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

    fn process_code(&mut self, host: &Host, interpreter: &mut Interpreter) {
        compile(&self.input.buffer, host, &mut self.code);

        self.input.clear();

        let is_running =
            matches!(interpreter.state(&self.code), InterpreterState::Running);

        if !is_running {
            interpreter.reset(&self.code);
        }
    }

    fn process_command(
        &mut self,
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
            self.error = Some(format!("Unknown command: `{command}`"));
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

            self.error = Some(error);

            return Ok(());
        }

        match matched_command {
            "clear" => {
                self.code = Code::default();
                interpreter.reset(&self.code);
            }
            "edit" => {
                self.mode = EditorMode::Edit;
            }
            "reset" => {
                interpreter.reset(&self.code);
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
    pub error: Option<&'r String>,
}
