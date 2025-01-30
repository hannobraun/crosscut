use crossterm::style::{Attribute, Color};

use crate::{
    io::terminal_editor::output::{Cursor, EditorOutputAdapter},
    language::{
        code::{Codebase, Expression, Location, Node},
        editor::Editor,
        host::Host,
        instance::Language,
        interpreter::Interpreter,
    },
};

use super::input::{EditorMode, TerminalEditorInput};

#[derive(Debug)]
pub struct TerminalEditorOutput<A> {
    adapter: A,
}

impl<A> TerminalEditorOutput<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn render(
        &mut self,
        language: &Language,
        editor_input: &TerminalEditorInput,
    ) -> anyhow::Result<()> {
        let mut context = RenderContext {
            codebase: language.codebase(),
            editor: language.editor(),
            interpreter: language.interpreter(),
            host: language.host(),
            cursor: None,
        };

        self.adapter.clear()?;

        render_code(&mut self.adapter, &mut context)?;
        render_prompt(&mut self.adapter, editor_input, &mut context)?;

        if let Some(Cursor { inner: [x, y] }) = context.cursor {
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
    writeln!(adapter)?;

    for (location, node) in context.codebase.nodes() {
        render_possibly_active_node(&location, node, adapter, context)?;
    }

    writeln!(adapter)?;

    Ok(())
}

fn render_possibly_active_node<A: EditorOutputAdapter>(
    location: &Location,
    node: &Node,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    if context.interpreter.state(context.codebase).location() == Some(location)
    {
        adapter.attribute(Attribute::Bold, |adapter| {
            render_node(location, node, adapter, context)
        })?;
    } else {
        render_node(location, node, adapter, context)?;
    }

    Ok(())
}

fn render_node<A: EditorOutputAdapter>(
    location: &Location,
    node: &Node,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    if context.editor.editing() == location {
        context.cursor =
            Some(adapter.cursor().move_right(context.editor.input().cursor()));
    }

    match node {
        Node::Empty => {
            writeln!(adapter)?;
        }
        Node::Expression { expression } => match expression {
            Expression::HostFunction { id } => {
                let name = context.host.function_name_by_id(id);
                adapter.color(Color::DarkMagenta, |adapter| {
                    writeln!(adapter, "{name}")
                })?;
            }
            Expression::IntrinsicFunction { function } => {
                adapter.color(Color::DarkBlue, |adapter| {
                    writeln!(adapter, "{function}")
                })?;
            }
        },
        Node::UnresolvedIdentifier { name } => {
            adapter.color(ERROR, |adapter| writeln!(adapter, "{name}"))?;
        }
    }

    Ok(())
}

fn render_prompt<A: EditorOutputAdapter>(
    adapter: &mut A,
    editor_input: &TerminalEditorInput,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    match editor_input.mode() {
        EditorMode::Edit => {
            writeln!(adapter, "Currently editing.")?;
            writeln!(adapter, "Press ESC to enter command mode.")?;
        }
        EditorMode::Command { input } => {
            write!(adapter, "Enter command > ")?;

            context.cursor = Some(adapter.cursor().move_right(input.cursor()));

            writeln!(adapter, "{}", input.buffer())?;
            writeln!(adapter, "Press ENTER to confirm, ESC to abort.")?;
        }
    }

    Ok(())
}

struct RenderContext<'r> {
    codebase: &'r Codebase,
    editor: &'r Editor,
    interpreter: &'r Interpreter,
    host: &'r Host,
    cursor: Option<Cursor>,
}

const ERROR: Color = Color::DarkRed;
