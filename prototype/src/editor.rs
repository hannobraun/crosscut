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

    pub fn process_input(&mut self, line: String) {
        let Some((command, input_code)) = line.trim().split_once(' ') else {
            println!(
                "Editor input must consist of a command and input code, \
                separated by whitespace."
            );
            return;
        };

        match command {
            ":insert" => {
                compile(input_code, &mut self.code);
            }
            _ => {
                println!("Unknown command: {command}");
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
    for fragment in &code.fragments {
        write!(w, "    ")?;

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
