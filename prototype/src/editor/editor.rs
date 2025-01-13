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

    pub fn process_input(
        &mut self,
        input: EditorInput,
        host: &Host,
        interpreter: &mut Interpreter,
    ) -> bool {
        match input {
            EditorInput::Char { value } => {
                self.input.push(value);
                return false;
            }
            EditorInput::Enter => {
                // Rest of function can do its thing now.
            }
        }

        let line = self.input.clone();
        self.input.clear();

        let mut command_and_arguments =
            line.trim().splitn(2, |ch: char| ch.is_whitespace());

        let Some(command) = command_and_arguments.next() else {
            return false;
        };
        let arguments = command_and_arguments.next();

        let mut matched_commands = self
            .commands
            .iter()
            .filter(|c| c.starts_with(command))
            .collect::<VecDeque<_>>();

        let Some(&matched_command) = matched_commands.pop_front() else {
            println!("Unknown command: `{command}`");
            return true;
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

            return true;
        }

        match matched_command {
            ":append" => {
                self.mode = EditorMode::Append;
            }
            command @ ":clear" => {
                let None = arguments else {
                    println!("`{command}` command expects no arguments.");
                    return true;
                };

                self.code = Code::default();
                interpreter.reset(&self.code);
            }
            command @ ":reset" => {
                let None = arguments else {
                    println!("`{command}` command expects no arguments.");
                    return true;
                };

                interpreter.reset(&self.code);
            }
            _ => {
                unreachable!("Ruled out that command is unknown, above.")
            }
        }

        if let EditorMode::Append = self.mode {
            if let Some(code) = arguments {
                self.process_code(code, host, interpreter);
            }

            self.mode = EditorMode::Command;
        }

        true
    }

    fn process_code(
        &mut self,
        code: &str,
        host: &Host,
        interpreter: &mut Interpreter,
    ) {
        for token in code.split_whitespace() {
            compile(token, host, &mut self.code);
        }

        let is_running =
            matches!(interpreter.state(&self.code), InterpreterState::Running);

        if !is_running {
            interpreter.reset(&self.code);
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
