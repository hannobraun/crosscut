use crossterm::style::{Attribute, Color};

use crate::{
    io::editor::output::EditorOutputAdapter,
    lang::{
        code::{
            Code, CodeError, Expression, FragmentError, FragmentKind,
            FunctionCallTarget, Literal, Located,
        },
        editor::{Editor, EditorError, EditorMode},
        host::Host,
        interpreter::{Interpreter, InterpreterState},
    },
};

use super::input::EditorInput;

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn print_code(code: &Code, host: &Host) {
    use crate::io::editor::output::DebugOutputAdapter;

    let mut adapter = DebugOutputAdapter;
    let mut context = RenderContext {
        code,
        host,
        editor: None,
        interpreter: None,
        indent: 0,
        cursor: None,
    };

    render_code(&mut adapter, &mut context).unwrap();
}

pub struct EditorOutput<A> {
    adapter: A,
}

impl<A> EditorOutput<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn render(
        &mut self,
        editor_input: &EditorInput,
        editor: &Editor,
        code: &Code,
        interpreter: &Interpreter,
        host: &Host,
    ) -> anyhow::Result<()> {
        let mut context = RenderContext {
            code,
            editor: Some((editor_input, editor)),
            interpreter: Some(interpreter),
            host,
            indent: 0,
            cursor: None,
        };

        self.adapter.clear()?;

        render_code(&mut self.adapter, &mut context)?;
        render_prompt(&mut self.adapter, editor_input, editor, &mut context)?;

        if let Some([x, y]) = context.cursor {
            self.adapter.move_cursor_to(x, y)?;
        }

        self.adapter.flush()?;

        Ok(())
    }
}

fn render_code<A: EditorOutputAdapter>(
    adapter: &mut A,
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

fn render_possibly_active_fragment<A: EditorOutputAdapter>(
    adapter: &mut A,
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

fn render_fragment<A: EditorOutputAdapter>(
    adapter: &mut A,
    located: Located,
    adjusted_indent: u32,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let maybe_error = context.code.errors.get(located.location.target());

    for _ in 0..adjusted_indent {
        render_indent(adapter)?;
    }

    let mut currently_editing_this_fragment = false;
    if let Some((_, editor)) = &context.editor {
        if editor.mode().is_edit() && editor.editing() == &located.location {
            currently_editing_this_fragment = true;

            context.cursor = {
                let [x, y] = adapter.cursor();
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

fn render_indent<A: EditorOutputAdapter>(
    adapter: &mut A,
) -> anyhow::Result<()> {
    write!(adapter, "    ")?;
    Ok(())
}

fn render_expression<A: EditorOutputAdapter>(
    adapter: &mut A,
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

            adapter.color(color, |adapter| write!(adapter, "{name}"))?;
        }
        Expression::Literal {
            literal: Literal::Integer { value },
        } => {
            write!(adapter, "{value}")?;
        }
    }

    Ok(())
}

fn render_prompt<A: EditorOutputAdapter>(
    adapter: &mut A,
    _: &EditorInput,
    editor: &Editor,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let mode = match editor.mode() {
        EditorMode::Command { .. } => "command",
        EditorMode::Edit { .. } => "edit",
    };

    if let Some(error) = editor.error() {
        writeln!(adapter)?;
        match error {
            EditorError::AmbiguousCommand {
                command,
                candidates,
            } => {
                writeln!(
                    adapter,
                    "`{command}` could refer to multiple commands:",
                )?;
                for candidate in candidates {
                    writeln!(adapter, "- `{candidate}`")?;
                }
            }
            EditorError::UnknownCommand { command } => {
                write!(adapter, "Unknown command: `{command}`")?;
            }
        }
    }

    writeln!(adapter)?;
    write!(adapter, "{mode} > ")?;

    match editor.mode() {
        EditorMode::Command { input } => {
            context.cursor = {
                let [x, y] = adapter.cursor();
                let x = {
                    let x: usize = x.into();
                    let x = x.saturating_add(input.cursor);
                    let x: u16 = x.try_into().unwrap_or(u16::MAX);
                    x
                };

                Some([x, y])
            };

            write!(adapter, "{}", input.buffer)?;
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
    editor: Option<(&'r EditorInput, &'r Editor)>,
    interpreter: Option<&'r Interpreter>,
    host: &'r Host,
    indent: u32,
    cursor: Option<[u16; 2]>,
}
