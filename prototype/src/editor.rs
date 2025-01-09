use std::io::{self, stdout};

use crossterm::{
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use crate::language::{
    code::{Code, Expression, Fragment, Token},
    compiler::compile,
    host::Host,
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
        host: &Host,
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

                compile(input_code, host, &mut self.code);
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

    pub fn render(
        &self,
        host: &Host,
        interpreter: &Interpreter,
    ) -> anyhow::Result<()> {
        let render = Render {
            code: &self.code,
            host,
            interpreter,
            w: stdout(),
        };
        render.render_code()?;
        Ok(())
    }
}

struct Render<'r, W> {
    code: &'r Code,
    host: &'r Host,
    interpreter: &'r Interpreter,
    w: W,
}

impl<W> Render<'_, W>
where
    W: io::Write,
{
    fn render_code(mut self) -> anyhow::Result<()> {
        writeln!(self.w)?;

        for (i, fragment) in self.code.fragments.iter().enumerate() {
            render_fragment(
                i,
                fragment,
                self.code,
                self.host,
                self.interpreter,
                &mut self.w,
            )?;
        }

        if self.interpreter.next_fragment == self.code.fragments.len() {
            writeln!(self.w, " => ")?;
        }

        writeln!(self.w)?;
        write!(self.w, "{} > ", self.interpreter.state(self.code))?;

        self.w.flush()?;

        Ok(())
    }
}

fn render_fragment(
    i: usize,
    fragment: &Fragment,
    code: &Code,
    host: &Host,
    interpreter: &Interpreter,
    mut w: impl io::Write,
) -> anyhow::Result<()> {
    if code.errors.contains(&i) {
        w.queue(SetForegroundColor(Color::Red))?;
    }

    if i == interpreter.next_fragment {
        w.queue(SetAttribute(Attribute::Bold))?;
        write!(w, " => ")?;
    } else {
        write!(w, "    ")?;
    }

    match fragment {
        Fragment::Expression { expression } => {
            render_expression(expression, host, &mut w)?;
        }
        Fragment::UnexpectedToken { token } => {
            match token {
                Token::Identifier { name } => {
                    write!(w, "{name}")?;
                }
                Token::LiteralNumber { value } => {
                    write!(w, "{value}")?;
                }
            }

            writeln!(w, "    error: unexpected token")?;
        }
    }

    w.queue(ResetColor)?;
    w.queue(SetAttribute(Attribute::Reset))?;

    Ok(())
}

fn render_expression(
    expression: &Expression,
    host: &Host,
    mut w: impl io::Write,
) -> anyhow::Result<()> {
    match expression {
        Expression::FunctionCall { target } => {
            let Some(name) = host.functions_by_id.get(target) else {
                unreachable!(
                    "Function call refers to non-existing function {target}"
                );
            };

            writeln!(w, "{name}")?;
        }
        Expression::LiteralValue { value } => {
            writeln!(w, "{value}")?;
        }
    }

    Ok(())
}
