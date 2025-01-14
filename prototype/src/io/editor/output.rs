use std::io::{stdout, Stdout, Write};

use crossterm::{
    cursor::{self, MoveToNextLine},
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand,
};

use crate::{
    editor::{Editor, EditorMode},
    language::{
        code::{
            Body, Code, CodeError, Expression, FragmentError, FragmentId,
            FragmentKind, Literal,
        },
        host::Host,
        interpreter::{Interpreter, InterpreterState},
    },
};

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn render_code(code: &Code, host: &Host) {
    let mut context = RenderContext {
        code,
        host,
        interpreter: None,
        indent: 0,
    };

    let mut renderer = Renderer::new().unwrap();
    renderer.render_code(&mut context).unwrap();
}

pub struct Renderer {
    w: Stdout,
}

impl Renderer {
    pub fn new() -> anyhow::Result<Self> {
        let mut w = stdout();

        // We render everything to the terminal's alternate screen. Entering the
        // alternate screen is undone in this type's `Drop` implementation.
        //
        // This way, we preserve the contents of the terminal as of before the
        // application was started. Just clearing those seems rude.
        w.queue(terminal::EnterAlternateScreen)?;

        // Nothing forces us to enable raw mode right here. It's also tied to
        // input, so we could enable it there.
        //
        // It is very important, however, that we _disable_ it consistently,
        // depending on where we enabled it. Otherwise the terminal will remain
        // in raw mode after the application exited.
        //
        // We are taking care of that here, by disabling raw mode in the `Drop`
        // implementation of this type. So raw mode is bound to its lifetime.
        terminal::enable_raw_mode()?;

        Ok(Self { w })
    }

    pub fn render(
        &mut self,
        editor: &Editor,
        host: &Host,
        interpreter: Option<&Interpreter>,
    ) -> anyhow::Result<()> {
        let mut context = RenderContext {
            code: editor.code(),
            host,
            interpreter,
            indent: 0,
        };

        self.w.queue(terminal::Clear(ClearType::All))?;
        self.w.queue(cursor::MoveTo(0, 0))?;

        self.render_code(&mut context)?;
        self.render_prompt(editor.mode(), editor.input())?;

        Ok(())
    }

    fn render_code(
        &mut self,
        context: &mut RenderContext,
    ) -> anyhow::Result<()> {
        if let Some(interpreter) = context.interpreter {
            let state = match interpreter.state(context.code) {
                InterpreterState::Running => "running",
                InterpreterState::Finished => "finished",
                InterpreterState::Error => "error",
            };

            write!(self.w, "process {state}")?;
            self.w.queue(MoveToNextLine(1))?;
        };

        self.render_fragment(&context.code.root, context)?;

        self.w.flush()?;

        Ok(())
    }

    fn render_prompt(
        &mut self,
        mode: &EditorMode,
        input: &String,
    ) -> anyhow::Result<()> {
        let mode = match mode {
            EditorMode::Append => "append",
            EditorMode::Command => "command",
        };

        self.w.queue(MoveToNextLine(1))?;
        write!(self.w, "{mode} > {input}")?;

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
        self.w.queue(MoveToNextLine(1))?;

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

impl Drop for Renderer {
    fn drop(&mut self) {
        // Nothing we can do about a potential error here.
        let _ = terminal::disable_raw_mode();
        let _ = self.w.queue(terminal::LeaveAlternateScreen);
    }
}

struct RenderContext<'r> {
    code: &'r Code,
    host: &'r Host,
    interpreter: Option<&'r Interpreter>,
    indent: u32,
}
