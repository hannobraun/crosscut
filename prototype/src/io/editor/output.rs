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
    let mut w = RawTerminalAdapter::new();
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

pub struct EditorOutput<A> {
    adapter: A,
}

impl EditorOutput<RawTerminalAdapter> {
    pub fn new() -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new();

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

        Ok(Self { adapter })
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

        self.adapter.clear()?;

        render_code(&mut self.adapter, &mut context)?;
        render_prompt(&mut self.adapter, editor, &mut context)?;

        if let Some([x, y]) = context.cursor {
            self.adapter.move_cursor_to(x, y)?;
        }

        self.adapter.flush()?;

        Ok(())
    }
}

impl<A> Drop for EditorOutput<A> {
    fn drop(&mut self) {
        // Nothing we can do about a potential error here.
        let _ = terminal::disable_raw_mode();
    }
}

fn render_code(
    adapter: &mut RawTerminalAdapter,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    if let Some(interpreter) = context.interpreter {
        let state = match interpreter.state(context.code) {
            InterpreterState::Running => "running",
            InterpreterState::Finished => "finished",
            InterpreterState::Error => "error",
        };

        writeln!(adapter, "process {state}")?;
    };

    render_possibly_active_fragment(adapter, context.code.root(), context)?;

    adapter.flush()?;

    Ok(())
}

fn render_possibly_active_fragment(
    adapter: &mut RawTerminalAdapter,
    located: Located,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let is_active = if let Some(interpreter) = context.interpreter {
        Some(located.location.target()) == interpreter.next()
    } else {
        false
    };

    let adjusted_indent = if is_active {
        // We're about to render an arrow for the active fragment, which is
        // worth one level of indentation. We need to adjust for that.
        let Some(adjusted_indent) = context.indent.checked_sub(1) else {
            unreachable!(
                "Every fragment body gets one level of indentation. The root \
                is a fragment. Hence, we must have at least one level of \
                indentation."
            );
        };

        adjusted_indent
    } else {
        context.indent
    };

    if is_active {
        adapter.attribute(Attribute::Bold, |adapter| {
            write!(adapter, " => ")?;
            render_fragment(adapter, located, adjusted_indent, context)?;

            Ok(())
        })?;
    } else {
        render_fragment(adapter, located, adjusted_indent, context)?;
    }

    Ok(())
}

fn render_fragment(
    adapter: &mut RawTerminalAdapter,
    located: Located,
    adjusted_indent: u32,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let maybe_error = context.code.errors.get(located.location.target());

    for _ in 0..adjusted_indent {
        render_indent(adapter)?;
    }

    let mut currently_editing_this_fragment = false;
    if let Some(editor) = &context.editor {
        if editor.mode().is_edit() && editor.editing() == &located.location {
            currently_editing_this_fragment = true;

            context.cursor = {
                let [x, y] = adapter.cursor;
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
                write!(adapter, "empty fragment")?;
            }
        }
        FragmentKind::Expression { expression } => {
            render_expression(adapter, expression, context)?;
        }
        FragmentKind::Error { err } => match err {
            FragmentError::IntegerOverflow { value } => {
                write!(adapter, "{value}")?;
            }
            FragmentError::MultiResolvedIdentifier { name } => {
                write!(adapter, "{name}")?;
            }
            FragmentError::UnresolvedIdentifier { name } => {
                write!(adapter, "{name}")?;
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

        adapter.color(Color::Red, |adapter| {
            write!(adapter, "    error: {message}")
        })?;
    }
    writeln!(adapter)?;

    context.indent += 1;
    for child in located.body(context.code.fragments()) {
        render_possibly_active_fragment(adapter, child, context)?;
    }
    context.indent -= 1;

    Ok(())
}

fn render_indent(w: &mut RawTerminalAdapter) -> anyhow::Result<()> {
    write!(w, "    ")?;
    Ok(())
}

fn render_expression(
    w: &mut RawTerminalAdapter,
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

            w.color(color, |w| write!(w, "{name}"))?;
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
    w: &mut RawTerminalAdapter,
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
                writeln!(w, "`{command}` could refer to multiple commands:",)?;
                for candidate in candidates {
                    writeln!(w, "- `{candidate}`")?;
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

trait EditorOutputAdapter: fmt::Write {
    fn clear(&mut self) -> io::Result<()>;

    fn write(&mut self, s: &str) -> io::Result<()>;

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()>;

    fn color(
        &mut self,
        color: Color,
        f: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> anyhow::Result<()>;

    fn attribute(
        &mut self,
        attribute: Attribute,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>;

    fn flush(&mut self) -> io::Result<()>;
}

/// # Adapter between the renderer and the terminal
///
/// Unfortunately, terminals are an ancient technology and suck very badly. As a
/// result, writing to the terminal directly turned out to be impractical.
///
/// The specific problem encountered, was that determining the cursor position
/// can't be done without causing a flush, which leads to visual artifacts when
/// then resuming the rendering. As a result, we at least need something to
/// track the cursor position throughout the render. Hence this adapter.
pub struct RawTerminalAdapter {
    w: Stdout,
    cursor: [u16; 2],
}

impl RawTerminalAdapter {
    pub fn new() -> Self {
        Self {
            w: stdout(),
            cursor: [0, 0],
        }
    }
}

impl EditorOutputAdapter for RawTerminalAdapter {
    fn clear(&mut self) -> io::Result<()> {
        self.w.queue(terminal::Clear(ClearType::All))?;
        self.move_cursor_to(0, 0)?;

        Ok(())
    }

    fn write(&mut self, s: &str) -> io::Result<()> {
        for ch in s.chars() {
            if ch == '\n' {
                if terminal::is_raw_mode_enabled()? {
                    self.w.queue(MoveToNextLine(1))?;
                } else {
                    // Terminal is not in raw mode, which means we're probably
                    // doing debug output. Don't mess around with commands, as
                    // to not interfere with other output that's possible being
                    // written around the same time.
                    writeln!(self.w)?;
                }

                self.cursor[0] = 0;
                self.cursor[1] += 1;
            } else {
                let mut buf = [0; 4];
                self.w.write_all(ch.encode_utf8(&mut buf).as_bytes())?;

                assert!(
                    ch.is_ascii(),
                    "Editor input adapter only accepts ASCII characters.",
                );
                self.cursor[0] += 1;
            }
        }

        Ok(())
    }

    fn move_cursor_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.w.queue(cursor::MoveTo(x, y))?;
        self.cursor = [x, y];
        Ok(())
    }

    fn color(
        &mut self,
        color: Color,
        f: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> anyhow::Result<()> {
        self.w.queue(SetForegroundColor(color))?;
        f(self)?;
        self.w.queue(ResetColor)?;

        Ok(())
    }

    fn attribute(
        &mut self,
        attribute: Attribute,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.w.queue(SetAttribute(attribute))?;
        f(self)?;
        self.w.queue(SetAttribute(Attribute::Reset))?;

        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

impl fmt::Write for RawTerminalAdapter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s).map_err(|_| fmt::Error)
    }
}
