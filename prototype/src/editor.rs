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
        Render {
            code: &self.code,
            host,
            interpreter,
            w: stdout(),
        }
        .render_code()?;

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
            render_fragment(&mut self, i, fragment)?;
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
    render: &mut Render<impl io::Write>,
    i: usize,
    fragment: &Fragment,
) -> anyhow::Result<()> {
    if render.code.errors.contains(&i) {
        render.w.queue(SetForegroundColor(Color::Red))?;
    }

    if i == render.interpreter.next_fragment {
        render.w.queue(SetAttribute(Attribute::Bold))?;
        write!(render.w, " => ")?;
    } else {
        write!(render.w, "    ")?;
    }

    match fragment {
        Fragment::Expression { expression } => {
            render_expression(expression, render.host, &mut render.w)?;
        }
        Fragment::UnexpectedToken { token } => {
            match token {
                Token::Identifier { name } => {
                    write!(render.w, "{name}")?;
                }
                Token::LiteralNumber { value } => {
                    write!(render.w, "{value}")?;
                }
            }

            writeln!(render.w, "    error: unexpected token")?;
        }
    }

    render.w.queue(ResetColor)?;
    render.w.queue(SetAttribute(Attribute::Reset))?;

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
