use std::io::{self, stdout, Stdout};

use crossterm::{
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use crate::language::{
    code::{
        Body, Code, CodeError, Expression, FragmentError, FragmentId,
        FragmentKind, Literal,
    },
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn render_code(
    code: &crate::language::code::Code,
    host: &crate::language::host::Host,
) {
    let mut renderer = Renderer::new(code, host, None);
    renderer.render_code().unwrap();
}

pub struct Renderer<'r, W> {
    w: W,
    context: RenderContext<'r>,
}

impl<'r> Renderer<'r, Stdout> {
    pub fn new(
        code: &'r Code,
        host: &'r Host,
        interpreter: Option<&'r Interpreter>,
    ) -> Self {
        Self {
            w: stdout(),
            context: RenderContext {
                code,
                host,
                interpreter,
                indent: 0,
            },
        }
    }
}

impl<W> Renderer<'_, W>
where
    W: io::Write,
{
    pub fn render(&mut self) -> anyhow::Result<()> {
        self.render_code()?;
        self.render_prompt()?;

        Ok(())
    }

    pub fn render_code(&mut self) -> anyhow::Result<()> {
        writeln!(self.w)?;
        self.render_fragment(&self.context.code.root)?;

        self.w.flush()?;

        Ok(())
    }

    fn render_prompt(&mut self) -> anyhow::Result<()> {
        let Some(interpreter) = self.context.interpreter else {
            unreachable!(
                "Rendering the prompt is only done in the full editor, where \
                the interpreter is available."
            );
        };

        let state = match interpreter.state(self.context.code) {
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
        let maybe_error = self.context.code.errors.get(id);

        if maybe_error.is_some() {
            self.w.queue(SetForegroundColor(Color::Red))?;
        }

        let mut indent = self.context.indent;
        if let Some(interpreter) = self.context.interpreter {
            if Some(id) == interpreter.next() {
                self.w.queue(SetAttribute(Attribute::Bold))?;
                write!(self.w, " => ")?;

                // This is worth one indentation level. We need to adjust for
                // that.
                let Some(adjusted) = self.context.indent.checked_sub(1) else {
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

        let fragment = self.context.code.fragments().get(id);

        match &fragment.kind {
            FragmentKind::Root => {
                // Nothing to render in the root fragment, except the body.
                // Which we're already doing below, unconditionally.
            }
            FragmentKind::Expression { expression } => {
                self.render_expression(expression)?;
            }
            FragmentKind::Error { err } => match err {
                FragmentError::IntegerOverflow { value } => {
                    write!(self.w, "{value}")?;
                }
                FragmentError::UnexpectedToken { token } => {
                    write!(self.w, "{token}")?;
                }
                FragmentError::UnresolvedIdentifier { name } => {
                    write!(self.w, "{name}")?;
                }
            },
        }

        if let Some(err) = maybe_error {
            let message = match err {
                CodeError::IntegerOverflow => "integer overflow",
                CodeError::MissingArgument => "missing argument",
                CodeError::UnexpectedToken => "unexpected token",
                CodeError::UnresolvedIdentifier => "unresolved identifier",
            };

            write!(self.w, "    error: {message}")?;
        }
        writeln!(self.w)?;

        self.context.indent += 1;
        self.render_body(&fragment.body)?;
        self.context.indent -= 1;

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
                let Some(name) = self.context.host.functions_by_id.get(target)
                else {
                    unreachable!(
                        "Function call refers to non-existing function {target}"
                    );
                };

                write!(self.w, "{name}")?;
            }
            Expression::Literal {
                literal: Literal::Integer { value },
            } => {
                write!(self.w, "{value}")?;
            }
        }

        Ok(())
    }
}

struct RenderContext<'r> {
    code: &'r Code,
    host: &'r Host,
    interpreter: Option<&'r Interpreter>,
    indent: u32,
}
