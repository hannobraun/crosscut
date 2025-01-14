use std::collections::{BTreeSet, VecDeque};

use crate::language::{
    code::Code,
    compiler::compile,
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

use super::EditorInput;

pub struct Editor {
    code: Code,
    mode: EditorMode,
    input: String,
    commands: BTreeSet<&'static str>,
}

impl Editor {
    pub fn new() -> Self {
        // All of the trie crates I could find where overly complex, unsuitable
        // for my use case, or dubious in other ways. Let's just do this by
        // hand.

        let mut commands = BTreeSet::new();
        commands.insert(":append");
        commands.insert(":clear");
        commands.insert(":reset");

        Self {
            code: Code::default(),
            mode: EditorMode::Command,
            input: String::new(),
            commands,
        }
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn input(&self) -> &String {
        &self.input
    }

    pub fn process_input(
        &mut self,
        input: EditorInput,
        host: &Host,
        interpreter: &mut Interpreter,
    ) -> Option<EditorTask> {
        match input {
            EditorInput::Char { value } => {
                self.input.push(value);
                Some(EditorTask::Render)
            }
            EditorInput::Enter => {
                match self.mode {
                    EditorMode::Append => {
                        self.process_code(host, interpreter);
                        self.mode = EditorMode::Command;
                    }
                    EditorMode::Command => {
                        self.process_command(interpreter);
                        self.input.clear();
                    }
                }

                Some(EditorTask::Render)
            }
        }
    }

    fn process_code(&mut self, host: &Host, interpreter: &mut Interpreter) {
        for token in self.input.split_whitespace() {
            compile(token, host, &mut self.code);
        }

        self.input.clear();

        let is_running =
            matches!(interpreter.state(&self.code), InterpreterState::Running);

        if !is_running {
            interpreter.reset(&self.code);
        }
    }

    fn process_command(&mut self, interpreter: &mut Interpreter) {
        let command = &self.input;

        let mut matched_commands = self
            .commands
            .iter()
            .filter(|c| c.starts_with(command))
            .collect::<VecDeque<_>>();

        let Some(&matched_command) = matched_commands.pop_front() else {
            println!("Unknown command: `{command}`");
            return;
        };
        if !matched_commands.is_empty() {
            print!(
                "`{command}` could refer to multiple commands: \
                `{matched_command}`"
            );
            for matched_command in matched_commands {
                print!(", `{matched_command}`");
            }
            println!();

            return;
        }

        match matched_command {
            ":append" => {
                self.mode = EditorMode::Append;
            }
            ":clear" => {
                self.code = Code::default();
                interpreter.reset(&self.code);
            }
            ":reset" => {
                interpreter.reset(&self.code);
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

#[derive(Debug)]
pub enum EditorMode {
    Append,
    Command,
}

pub enum EditorTask {
    Render,
}
