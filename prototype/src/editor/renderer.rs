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
pub fn render_code(code: &Code, host: &Host) {
    let mut context = RenderContext {
        code,
        host,
        interpreter: None,
        indent: 0,
    };

    let mut renderer = Renderer::new();
    renderer.render_code(&mut context).unwrap();
}

pub struct Renderer<W> {
    w: W,
}

impl Renderer<Stdout> {
    pub fn new() -> Self {
        Self { w: stdout() }
    }
}

impl<W> Renderer<W>
where
    W: io::Write,
{
    pub fn render(
        &mut self,
        code: &Code,
        host: &Host,
        interpreter: Option<&Interpreter>,
    ) -> anyhow::Result<()> {
        let mut context = RenderContext {
            code,
            host,
            interpreter,
            indent: 0,
        };

        self.render_code(&mut context)?;
        self.render_prompt(&context)?;

        Ok(())
    }

    fn render_code(
        &mut self,
        context: &mut RenderContext,
    ) -> anyhow::Result<()> {
        writeln!(self.w)?;
        self.render_fragment(&context.code.root, context)?;

        self.w.flush()?;

        Ok(())
    }

    fn render_prompt(&mut self, context: &RenderContext) -> anyhow::Result<()> {
        let Some(interpreter) = context.interpreter else {
            unreachable!(
                "Rendering the prompt is only done in the full editor, where \
                the interpreter is available."
            );
        };

        let state = match interpreter.state(context.code) {
            InterpreterState::Running => "running",
            InterpreterState::Finished => "finished",
            InterpreterState::Error => "error",
        };

        writeln!(self.w)?;
        write!(self.w, "{state} > ")?;

        self.w.flush()?;

        Ok(())
    }

    fn render_body(
        &mut self,
        body: &Body,
        context: &mut RenderContext,
    ) -> anyhow::Result<()> {
        for hash in body.ids() {
            self.render_fragment(hash, context)?;
        }

        Ok(())
    }

    fn render_fragment(
        &mut self,
        id: &FragmentId,
        context: &mut RenderContext,
    ) -> anyhow::Result<()> {
        let maybe_error = context.code.errors.get(id);

        if maybe_error.is_some() {
            self.w.queue(SetForegroundColor(Color::Red))?;
        }

        let mut indent = context.indent;
        if let Some(interpreter) = context.interpreter {
            if Some(id) == interpreter.next() {
                self.w.queue(SetAttribute(Attribute::Bold))?;
                write!(self.w, " => ")?;

                // This is worth one indentation level. We need to adjust for
                // that.
                let Some(adjusted) = context.indent.checked_sub(1) else {
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

        let fragment = context.code.fragments().get(id);

        match &fragment.kind {
            FragmentKind::Root => {
                // Nothing to render in the root fragment, except the body.
                // Which we're already doing below, unconditionally.
            }
            FragmentKind::Expression { expression } => {
                self.render_expression(expression, context)?;
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

        context.indent += 1;
        self.render_body(&fragment.body, context)?;
        context.indent -= 1;

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
        context: &RenderContext,
    ) -> anyhow::Result<()> {
        match expression {
            Expression::FunctionCall { target } => {
                let Some(name) = context.host.functions_by_id.get(target)
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
