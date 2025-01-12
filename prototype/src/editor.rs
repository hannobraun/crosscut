use std::{
    collections::{BTreeSet, VecDeque},
    io::{self, stdout, Stdout},
};

use crossterm::{
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use crate::language::{
    code::{
        Body, Code, CodeError, Expression, FragmentError, FragmentId,
        FragmentKind, Token,
    },
    compiler::compile,
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

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
        line: String,
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

pub struct Renderer<'r, W> {
    code: &'r Code,
    host: &'r Host,
    interpreter: Option<&'r Interpreter>,
    w: W,
    indent: u32,
}

impl<'r> Renderer<'r, Stdout> {
    pub fn new(
        code: &'r Code,
        host: &'r Host,
        interpreter: Option<&'r Interpreter>,
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
        self.render_fragment(&self.code.root)?;

        self.w.flush()?;

        Ok(())
    }

    pub fn render_prompt(&mut self) -> anyhow::Result<()> {
        let Some(interpreter) = self.interpreter else {
            unreachable!(
                "Rendering the prompt is only done in the full editor, where \
                the interpreter is available."
            );
        };

        let state = match interpreter.state(self.code) {
            InterpreterState::Running => "running",
            InterpreterState::Finished => "finished",
            InterpreterState::Error => "error",
        };

        writeln!(self.w)?;
        write!(self.w, "{state} > ")?;

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
        let maybe_error = self.code.errors.get(id);

        if maybe_error.is_some() {
            self.w.queue(SetForegroundColor(Color::Red))?;
        }

        let mut indent = self.indent;
        if let Some(interpreter) = self.interpreter {
            if Some(id) == interpreter.next() {
                self.w.queue(SetAttribute(Attribute::Bold))?;
                write!(self.w, " => ")?;

                // This is worth one indentation level. We need to adjust for
                // that.
                let Some(adjusted) = self.indent.checked_sub(1) else {
                    unreachable!(
                        "Every fragment body gets one level of indentation. \
                        The root is a fragment. Hence, we must have at least \
                        one level of indentation."
                    );
                };
                indent = adjusted;
            }
        };

        for _ in 0..indent {
            self.render_indent()?;
        }

        let fragment = self.code.fragments().get(id);

        match &fragment.kind {
            FragmentKind::Root => {
                // Nothing to render in the root fragment, except the body.
                // Which we're already doing below, unconditionally.
            }
            FragmentKind::Expression { expression } => {
                self.render_expression(expression)?;
            }
            FragmentKind::Error { err } => match err {
                FragmentError::UnexpectedToken { token } => match token {
                    Token::Identifier { name } => {
                        write!(self.w, "{name}")?;
                    }
                    Token::LiteralInteger { value } => {
                        write!(self.w, "{value}")?;
                    }
                },
                FragmentError::UnresolvedIdentifier { name } => {
                    write!(self.w, "{name}")?;
                }
            },
        }

        if let Some(err) = maybe_error {
            let message = match err {
                CodeError::MissingArgument => "missing argument",
                CodeError::UnexpectedToken => "unexpected token",
                CodeError::UnresolvedIdentifier => "unresolved identifier",
            };

            write!(self.w, "    error: {message}")?;
        }
        writeln!(self.w)?;

        self.indent += 1;
        self.render_body(&fragment.body)?;
        self.indent -= 1;

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

                write!(self.w, "{name}")?;
            }
            Expression::LiteralInteger { value } => {
                write!(self.w, "{value}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn render_code(code: &Code, host: &Host) {
    let mut renderer = Renderer::new(code, host, None);
    renderer.render_code().unwrap();
}
