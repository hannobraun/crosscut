use std::io::{self, stdout};

use crossterm::{
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use crate::language::{
    code::{Body, Code, Expression, Fragment, FragmentId, Token},
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
                interpreter.reset(&self.code);
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

                interpreter.reset(&self.code);
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

        self.render_body(&self.code.root)?;

        writeln!(self.w)?;
        write!(self.w, "{} > ", self.interpreter.state(self.code))?;

        self.w.flush()?;

        Ok(())
    }

    fn render_body(&mut self, body: &Body) -> anyhow::Result<()> {
        for hash in body.ids() {
            self.render_fragment(hash)?;
        }

        Ok(())
    }

    fn render_fragment(&mut self, id: &FragmentId) -> anyhow::Result<()> {
        if self.code.errors.contains(id) {
            self.w.queue(SetForegroundColor(Color::Red))?;
        }

        if Some(id) == self.interpreter.next.as_ref() {
            self.w.queue(SetAttribute(Attribute::Bold))?;
            write!(self.w, " => ")?;
        } else {
            write!(self.w, "    ")?;
        }

        match self.code.fragments().get(id) {
            Fragment::Expression { expression } => {
                self.render_expression(expression)?;
            }
            Fragment::UnexpectedToken { token } => {
                match token {
                    Token::Identifier { name } => {
                        write!(self.w, "{name}")?;
                    }
                    Token::LiteralNumber { value } => {
                        write!(self.w, "{value}")?;
                    }
                }

                writeln!(self.w, "    error: unexpected token")?;
            }
        }

        self.w.queue(ResetColor)?;
        self.w.queue(SetAttribute(Attribute::Reset))?;

        Ok(())
    }

    fn render_expression(
        &mut self,
        expression: &Expression,
    ) -> anyhow::Result<()> {
        match expression {
            Expression::FunctionCall { target, argument } => {
                let Some(name) = self.host.functions_by_id.get(target) else {
                    unreachable!(
                        "Function call refers to non-existing function {target}"
                    );
                };

                writeln!(self.w, "{name}")?;
                self.render_body(argument)?;
            }
            Expression::LiteralValue { value } => {
                writeln!(self.w, "{value}")?;
            }
        }

        Ok(())
    }
}
