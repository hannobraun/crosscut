use std::collections::{BTreeSet, VecDeque};

use crate::language::{
    code::Code,
    compiler::compile,
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

use super::{EditorInput, Renderer};

pub struct Editor {
    code: Code,
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
            commands,
        }
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn process_input(
        &mut self,
        EditorInput { line }: EditorInput,
        host: &Host,
        interpreter: &mut Interpreter,
    ) {
        let mut command_and_arguments =
            line.trim().splitn(2, |ch: char| ch.is_whitespace());

        let Some(command) = command_and_arguments.next() else {
            return;
        };

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
            command @ ":append" => {
                let Some(input_code) = command_and_arguments.next() else {
                    println!(
                        "`{command}` command expects input code as argument."
                    );
                    return;
                };

                compile(input_code, host, &mut self.code);

                let is_running = matches!(
                    interpreter.state(&self.code),
                    InterpreterState::Running
                );

                if !is_running {
                    interpreter.reset(&self.code);
                }
            }
            command @ ":clear" => {
                let None = command_and_arguments.next() else {
                    println!("`{command}` command expects no arguments.");
                    return;
                };

                self.code = Code::default();
                interpreter.reset(&self.code);
            }
            command @ ":reset" => {
                let None = command_and_arguments.next() else {
                    println!("`{command}` command expects no arguments.");
                    return;
                };

                interpreter.reset(&self.code);
            }
            _ => {
                unreachable!("Ruled out that command is unknown, above.")
            }
        }
    }

    pub fn render(
        &self,
        host: &Host,
        interpreter: &Interpreter,
    ) -> anyhow::Result<()> {
        let mut renderer = Renderer::new(&self.code, host, Some(interpreter));

        renderer.render_code()?;
        renderer.render_prompt()?;

        Ok(())
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
