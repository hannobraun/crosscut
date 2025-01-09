use std::io::{self, stdout};

use crate::language::{
    code::{Code, Expression, Fragment, Token},
    compiler::compile,
    interpreter::Interpreter,
};

#[derive(Default)]
pub struct Editor {
    code: Code,
}

impl Editor {
    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn process_input(
        &mut self,
        line: String,
        interpreter: &mut Interpreter,
    ) {
        let mut command_and_arguments =
            line.trim().splitn(2, |ch: char| ch.is_whitespace());

        let Some(command) = command_and_arguments.next() else {
            return;
        };

        match command {
            command @ ":clear" => {
                let None = command_and_arguments.next() else {
                    println!("`{command}` command expects no arguments.");
                    return;
                };

                self.code = Code::default();
            }
            command @ ":insert" => {
                let Some(input_code) = command_and_arguments.next() else {
                    println!(
                        "`{command}` command expects input code as argument."
                    );
                    return;
                };

                compile(input_code, &mut self.code);
            }
            command @ ":reset" => {
                let None = command_and_arguments.next() else {
                    println!("`{command}` command expects no arguments.");
                    return;
                };

                *interpreter = Interpreter::default();
            }
            command => {
                println!("Unknown command: `{command}`");
            }
        }
    }

    pub fn render(&self, interpreter: &Interpreter) -> anyhow::Result<()> {
        render_code(&self.code, interpreter, stdout())?;
        Ok(())
    }
}

fn render_code(
    code: &Code,
    interpreter: &Interpreter,
    mut w: impl io::Write,
) -> anyhow::Result<()> {
    writeln!(w)?;

    for (i, fragment) in code.fragments.iter().enumerate() {
        if i == interpreter.next_fragment {
            write!(w, " => ")?;
        } else {
            write!(w, "    ")?;
        }

        match fragment {
            Fragment::Expression { expression } => match expression {
                Expression::LiteralValue { value } => {
                    writeln!(w, "{value}")?;
                }
            },
            Fragment::UnexpectedToken { token } => match token {
                Token::Identifier { name } => {
                    writeln!(w, "{name}")?;
                }
                Token::LiteralNumber { value } => {
                    writeln!(w, "{value}")?;
                }
            },
        }
    }

    writeln!(w)?;
    write!(w, "{} > ", interpreter.state(code))?;

    w.flush()?;

    Ok(())
}
