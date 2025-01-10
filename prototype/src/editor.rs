use std::io::{self, stdout, Stdout};

use crossterm::{
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use crate::language::{
    code::{Body, Code, Expression, FragmentId, FragmentKind, Token},
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
        let mut renderer = Renderer::new(&self.code, host, interpreter);

        renderer.render_code()?;
        renderer.render_prompt()?;

        Ok(())
    }
}

pub struct Renderer<'r, W> {
    code: &'r Code,
    host: &'r Host,
    interpreter: &'r Interpreter,
    w: W,
    indent: u32,
}

impl<'r> Renderer<'r, Stdout> {
    pub fn new(
        code: &'r Code,
        host: &'r Host,
        interpreter: &'r Interpreter,
    ) -> Self {
        Self {
            code,
            host,
            interpreter,
            w: stdout(),
            indent: 0,
        }
    }
}

impl<W> Renderer<'_, W>
where
    W: io::Write,
{
    pub fn render_code(&mut self) -> anyhow::Result<()> {
        writeln!(self.w)?;
        self.render_body(&self.code.root)?;

        self.w.flush()?;

        Ok(())
    }

    pub fn render_prompt(&mut self) -> anyhow::Result<()> {
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
            self.render_indent()?;
        }

        for _ in 0..self.indent {
            self.render_indent()?;
        }

        let fragment = self.code.fragments().get(id);

        match &fragment.kind {
            FragmentKind::Expression { expression } => {
                self.render_expression(expression)?;
                self.render_body(&fragment.body)?;
            }
            FragmentKind::UnexpectedToken { token } => {
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

    fn render_indent(&mut self) -> anyhow::Result<()> {
        write!(self.w, "    ")?;
        Ok(())
    }

    fn render_expression(
        &mut self,
        expression: &Expression,
    ) -> anyhow::Result<()> {
        match expression {
            Expression::FunctionCall { target } => {
                let Some(name) = self.host.functions_by_id.get(target) else {
                    unreachable!(
                        "Function call refers to non-existing function {target}"
                    );
                };

                writeln!(self.w, "{name}")?;
            }
            Expression::LiteralValue { value } => {
                writeln!(self.w, "{value}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn render_code(code: &Code, host: &Host, interpreter: &Interpreter) {
    let mut renderer = Renderer::new(code, host, interpreter);
    renderer.render_code().unwrap();
}
