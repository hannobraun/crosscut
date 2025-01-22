use std::{
    fmt::{self, Write as _},
    io::{self, stdout, Stdout, Write as _},
};

use crossterm::{
    cursor::{self, MoveToNextLine},
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
    QueueableCommand,
};

use crate::lang::{
    code::{
        Code, CodeError, Expression, FragmentError, FragmentKind,
        FunctionCallTarget, Literal, Located,
    },
    editor::{Editor, EditorError, EditorMode},
    host::Host,
    interpreter::{Interpreter, InterpreterState},
};

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn print_code(code: &Code, host: &Host) {
    let mut w = EditorOutputAdapter::new();
    let mut context = RenderContext {
        code,
        host,
        editor: None,
        interpreter: None,
        indent: 0,
        cursor: None,
    };

    render_code(&mut w, &mut context).unwrap();
}

pub struct EditorOutput {
    w: EditorOutputAdapter,
}

impl EditorOutput {
    pub fn new() -> anyhow::Result<Self> {
        let w = EditorOutputAdapter::new();

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
        code: &Code,
        interpreter: &Interpreter,
        host: &Host,
    ) -> anyhow::Result<()> {
        let mut context = RenderContext {
            code,
            editor: Some(editor),
            interpreter: Some(interpreter),
            host,
            indent: 0,
            cursor: None,
        };

        self.w.clear()?;
        self.w.move_to(0, 0)?;

        render_code(&mut self.w, &mut context)?;
        render_prompt(&mut self.w, editor, &mut context)?;

        if let Some([x, y]) = context.cursor {
            self.w.move_to(x, y)?;
        }

        self.w.flush()?;

        Ok(())
    }
}

impl Drop for EditorOutput {
    fn drop(&mut self) {
        // Nothing we can do about a potential error here.
        let _ = terminal::disable_raw_mode();
    }
}

fn render_code(
    w: &mut EditorOutputAdapter,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    if let Some(interpreter) = context.interpreter {
        let state = match interpreter.state(context.code) {
            InterpreterState::Running => "running",
            InterpreterState::Finished => "finished",
            InterpreterState::Error => "error",
        };

        write!(w, "process {state}")?;
        writeln!(w)?;
    };

    render_fragment(w, context.code.root(), context)?;

    w.flush()?;

    Ok(())
}

fn render_fragment(
    w: &mut EditorOutputAdapter,
    located: Located,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let maybe_error = context.code.errors.get(located.location.target());

    if maybe_error.is_some() {
        w.set_foreground_color(Color::Red)?;
    }

    let mut indent = context.indent;
    if let Some(interpreter) = context.interpreter {
        if Some(located.location.target()) == interpreter.next() {
            w.set_attribute(Attribute::Bold)?;
            write!(w, " => ")?;

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
        render_indent(w)?;
    }

    let mut currently_editing_this_fragment = false;
    if let Some(editor) = &context.editor {
        if editor.mode().is_edit() && editor.editing() == &located.location {
            currently_editing_this_fragment = true;

            context.cursor = {
                let [x, y] = w.cursor;
                let x = {
                    let x: usize = x.into();
                    let x = x.saturating_add(editor.input().cursor);
                    let x: u16 = x.try_into().unwrap_or(u16::MAX);
                    x
                };

                Some([x, y])
            };
        }
    }

    match &located.fragment.kind {
        FragmentKind::Root => {
            // Nothing to render in the root fragment, except the body.
            // Which we're already doing below, unconditionally.
        }
        FragmentKind::Empty => {
            if currently_editing_this_fragment {
                // We're already drawing the cursor right here. Drawing anything
                // else for an empty fragment is only going to interfere with
                // that.
            } else {
                write!(w, "empty fragment")?;
            }
        }
        FragmentKind::Expression { expression } => {
            render_expression(w, expression, context)?;
        }
        FragmentKind::Error { err } => match err {
            FragmentError::IntegerOverflow { value } => {
                write!(w, "{value}")?;
            }
            FragmentError::MultiResolvedIdentifier { name } => {
                write!(w, "{name}")?;
            }
            FragmentError::UnresolvedIdentifier { name } => {
                write!(w, "{name}")?;
            }
        },
    }

    if let Some(err) = maybe_error {
        let message = match err {
            CodeError::IntegerOverflow => "integer overflow",
            CodeError::MissingArgument => "missing argument",
            CodeError::MultiResolvedIdentifier => {
                "identifier resolved multiple times"
            }
            CodeError::UnexpectedToken => "unexpected token",
            CodeError::UnresolvedIdentifier => "unresolved identifier",
        };

        write!(w, "    error: {message}")?;
    }
    writeln!(w)?;

    context.indent += 1;
    for child in located.body(context.code.fragments()) {
        render_fragment(w, child, context)?;
    }
    context.indent -= 1;

    w.reset_color()?;
    w.set_attribute(Attribute::Reset)?;

    Ok(())
}

fn render_indent(w: &mut EditorOutputAdapter) -> anyhow::Result<()> {
    write!(w, "    ")?;
    Ok(())
}

fn render_expression(
    w: &mut EditorOutputAdapter,
    expression: &Expression,
    context: &RenderContext,
) -> anyhow::Result<()> {
    match expression {
        Expression::FunctionCall { target } => {
            let (color, name) = match target {
                FunctionCallTarget::HostFunction { id } => {
                    let color = Color::DarkMagenta;

                    let Some(name) = context.host.functions_by_id.get(id)
                    else {
                        unreachable!(
                            "Function call refers to non-existing host \
                            function `{id}`"
                        );
                    };

                    (color, name.as_str())
                }
                FunctionCallTarget::IntrinsicFunction => {
                    (Color::DarkBlue, "identity")
                }
            };

            w.queue(SetForegroundColor(color))?;
            write!(w, "{name}")?;
            w.queue(ResetColor)?;
        }
        Expression::Literal {
            literal: Literal::Integer { value },
        } => {
            write!(w, "{value}")?;
        }
    }

    Ok(())
}

fn render_prompt(
    w: &mut EditorOutputAdapter,
    editor: &Editor,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let mode = match editor.mode() {
        EditorMode::Command { .. } => "command",
        EditorMode::Edit { .. } => "edit",
    };

    if let Some(error) = editor.error() {
        writeln!(w)?;
        match error {
            EditorError::AmbiguousCommand {
                command,
                candidates,
            } => {
                write!(w, "`{command}` could refer to multiple commands:",)?;
                writeln!(w)?;
                for candidate in candidates {
                    write!(w, "- `{candidate}`")?;
                    writeln!(w)?;
                }
            }
            EditorError::UnknownCommand { command } => {
                write!(w, "Unknown command: `{command}`")?;
            }
        }
    }

    writeln!(w)?;
    write!(w, "{mode} > ")?;

    match editor.mode() {
        EditorMode::Command { input } => {
            context.cursor = {
                let [x, y] = w.cursor;
                let x = {
                    let x: usize = x.into();
                    let x = x.saturating_add(input.cursor);
                    let x: u16 = x.try_into().unwrap_or(u16::MAX);
                    x
                };

                Some([x, y])
            };

            write!(w, "{}", input.buffer)?;
        }
        EditorMode::Edit { .. } => {
            // If we're in edit mode, the editing happens directly where the
            // code is displayed, and there's no need to display any input here.
        }
    }

    Ok(())
}

struct RenderContext<'r> {
    code: &'r Code,
    editor: Option<&'r Editor>,
    interpreter: Option<&'r Interpreter>,
    host: &'r Host,
    indent: u32,
    cursor: Option<[u16; 2]>,
}

/// # Adapter between the renderer and the terminal
///
/// Unfortunately, terminals are an ancient technology and suck very badly. As a
/// result, writing to the terminal directly turned out to be impractical.
///
/// The specific problem encountered, was that determining the cursor position
/// can't be read without causing a flush, which leads to visual artifacts when
/// then resuming the rendering. As a result, we at least need something to
/// track the cursor position throughout the render. Hence this adapter.
///
/// ## Implementation Note
///
/// The API of this type leaves something to be desired. It was initially
/// created to support the existing (Crossterm-based) usage patterns.
struct EditorOutputAdapter {
    w: Stdout,
    cursor: [u16; 2],
}

impl EditorOutputAdapter {
    fn new() -> Self {
        Self {
            w: stdout(),
            cursor: [0, 0],
        }
    }

    fn clear(&mut self) -> anyhow::Result<()> {
        self.w.queue(terminal::Clear(ClearType::All))?;
        Ok(())
    }

    fn move_to(&mut self, x: u16, y: u16) -> anyhow::Result<()> {
        self.w.queue(cursor::MoveTo(x, y))?;
        self.cursor = [x, y];
        Ok(())
    }

    fn move_to_next_line(&mut self) -> anyhow::Result<()> {
        if terminal::is_raw_mode_enabled()? {
            self.w.queue(MoveToNextLine(1))?;
        } else {
            // Terminal is not in raw mode, which means we're probably doing
            // debug output. Don't mess around with commands, as to not
            // interfere with other output that's possible being written around
            // the same time.
            writeln!(self.w)?;
        }

        self.cursor = {
            let [_, y] = self.cursor;
            [0, y + 1]
        };

        Ok(())
    }

    fn set_foreground_color(&mut self, color: Color) -> anyhow::Result<()> {
        self.w.queue(SetForegroundColor(color))?;
        Ok(())
    }

    fn set_attribute(&mut self, attribute: Attribute) -> anyhow::Result<()> {
        self.w.queue(SetAttribute(attribute))?;
        Ok(())
    }

    fn reset_color(&mut self) -> anyhow::Result<()> {
        self.w.queue(ResetColor)?;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // We're only accepting ASCII characters from the terminal right now, so
        // this should work fine.
        let bytes_written = self.w.write(buf)?;
        self.cursor[0] += bytes_written as u16;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }

    fn queue(
        &mut self,
        command: impl crossterm::Command,
    ) -> anyhow::Result<()> {
        self.w.queue(command)?;
        Ok(())
    }
}

impl fmt::Write for EditorOutputAdapter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            if ch == '\n' {
                self.move_to_next_line()
                    .map(|_| ())
                    .map_err(|_| fmt::Error)?;
            } else {
                let mut buf = [0; 4];
                self.write(ch.encode_utf8(&mut buf).as_bytes())
                    .map(|_| ())
                    .map_err(|_| fmt::Error)?;
            }
        }

        Ok(())
    }
}
