use crossterm::style::{Attribute, Color};

use crate::{
    io::editor::output::{Cursor, EditorOutputAdapter},
    language::{
        code::{Codebase, NodePath, SyntaxNode},
        editor::{Editor, EditorLayout, EditorLine},
        language::Language,
        runtime::{Effect, Evaluator, RuntimeState},
    },
};

use super::input::{EditorMode, TerminalEditorInput};

#[cfg(test)]
pub fn codebase_to_stdout(codebase: &Codebase) {
    use crate::io::editor::output::DebugOutputAdapter;
    codebase_to_adapter(codebase, &mut DebugOutputAdapter);
}

#[cfg(test)]
pub fn codebase_to_string(codebase: &Codebase) -> String {
    use crate::io::editor::output::StringOutputAdapter;

    let mut adapter = StringOutputAdapter {
        output: String::new(),
    };
    codebase_to_adapter(codebase, &mut adapter);

    adapter.output
}

#[cfg(test)]
fn codebase_to_adapter(
    codebase: &Codebase,
    adapter: &mut impl EditorOutputAdapter,
) {
    let layout = EditorLayout::new(codebase.root(), codebase);

    let mut context = RenderContext {
        codebase,
        editor: None,
        evaluator: None,
        cursor: None,
    };

    render_layout(&layout, adapter, &mut context)
        .expect("Failed to render code")
}

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
            editor: Some(language.editor()),
            evaluator: Some(language.evaluator()),
            cursor: None,
        };

        let layout =
            EditorLayout::new(context.codebase.root(), context.codebase);

        self.adapter.clear()?;

        render_runtime_state(&mut self.adapter, &context)?;
        render_layout(&layout, &mut self.adapter, &mut context)?;
        render_prompt(&mut self.adapter, editor_input, &mut context)?;
        render_help(&mut self.adapter, &context)?;

        if let Some(cursor) = context.cursor {
            self.adapter.move_cursor_to(cursor.position)?;
        }

        self.adapter.flush()?;

        Ok(())
    }
}

fn render_runtime_state<A: EditorOutputAdapter>(
    adapter: &mut A,
    context: &RenderContext,
) -> anyhow::Result<()> {
    let Some(evaluator) = context.evaluator else {
        return Ok(());
    };

    adapter.attribute(Attribute::Bold, |adapter| {
        match evaluator.state() {
            RuntimeState::Started | RuntimeState::Running => {
                adapter.color(Color::DarkGreen, |adapter| {
                    writeln!(adapter, "Running")?;
                    Ok(())
                })?;
            }
            RuntimeState::Effect { effect, .. } => {
                adapter.color(Color::DarkCyan, |adapter| {
                    write!(adapter, "Effect: ")?;

                    match effect {
                        Effect::ApplyProvidedFunction { name, input } => {
                            writeln!(
                                adapter,
                                "applying provided function `{name}` (input: \
                                {input})",
                            )?;
                        }
                        Effect::ProvidedFunctionNotFound => {
                            writeln!(adapter, "provided function not found")?;
                        }
                        Effect::UnexpectedInput { expected, actual } => {
                            writeln!(
                                adapter,
                                "unexpected input (expected `{expected}`, got \
                                `{actual}`)"
                            )?;
                        }
                    }

                    Ok(())
                })?;
            }
            RuntimeState::Finished { output } => {
                adapter.color(Color::DarkYellow, |adapter| {
                    writeln!(adapter, "Finished: {output}")?;
                    Ok(())
                })?;
            }
        }

        Ok(())
    })?;

    Ok(())
}

fn render_layout<A: EditorOutputAdapter>(
    layout: &EditorLayout,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    writeln!(adapter)?;

    for line in layout.lines.iter() {
        render_possibly_active_line(line, adapter, context)?;
    }

    writeln!(adapter)?;

    Ok(())
}

fn render_possibly_active_line<A: EditorOutputAdapter>(
    line: &EditorLine,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let is_active_node = if let Some(evaluator) = context.evaluator {
        evaluator.state().path() == Some(&line.node.path)
    } else {
        false
    };

    if is_active_node {
        adapter.attribute(Attribute::Bold, |adapter| {
            write!(adapter, " => ")?;
            render_line(line, adapter, context)
        })?;
    } else {
        write!(adapter, "    ")?;
        render_line(line, adapter, context)?;
    }

    writeln!(adapter)?;

    Ok(())
}

fn render_line<A: EditorOutputAdapter>(
    line: &EditorLine,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    for _ in 0..line.width_of_indentation() {
        adapter.color(Color::Grey, |adapter| {
            write!(adapter, "·")?;
            Ok(())
        })?;
    }

    render_node(&line.node.path, adapter, context)?;

    Ok(())
}

fn render_node<A: EditorOutputAdapter>(
    path: &NodePath,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let node = context.codebase.node_at(path).node;

    if let Some(editor) = context.editor {
        if &editor.cursor().path == path {
            context.cursor =
                Some(adapter.cursor().move_right(editor.cursor().index));
        }
    }

    let color = match node {
        SyntaxNode::Identifier { .. } => Some(Color::DarkMagenta),
        SyntaxNode::Function { .. }
        | SyntaxNode::Number { .. }
        | SyntaxNode::Tuple { .. } => Some(Color::DarkBlue),
        _ => None,
    };

    if let Some(color) = color {
        adapter.color(color, |adapter| {
            write!(adapter, "{node}")?;
            Ok(())
        })?;
    } else {
        write!(adapter, "{node}")?;
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
        EditorMode::Command { input, cursor } => {
            write!(adapter, "Enter command > ")?;

            context.cursor = Some(adapter.cursor().move_right(*cursor));

            writeln!(adapter, "{}", input.buffer())?;
            writeln!(adapter, "Press ENTER to confirm, ESC to abort.")?;
        }
    }

    Ok(())
}

// This help system is rather rudimentary. Just a first draft. But it could
// serve as the foundation of a more full-featured help browser, which both
// serves as a kind of tutorial, but also provides reference material.
//
// This is tracked in this issue:
// https://github.com/hannobraun/crosscut/issues/67
fn render_help<A: EditorOutputAdapter>(
    adapter: &mut A,
    context: &RenderContext,
) -> anyhow::Result<()> {
    let Some(editor) = context.editor else {
        return Ok(());
    };

    let node = context.codebase.node_at(&editor.cursor().path).node;

    writeln!(adapter)?;

    match node {
        SyntaxNode::Add => {
            writeln!(
                adapter,
                "Edit this node to add a new child to the parent.",
            )?;
        }
        SyntaxNode::Apply { .. } => {
            writeln!(
                adapter,
                "This is the application of a function to an argument.",
            )?;
        }
        SyntaxNode::Binding { .. } => {
            writeln!(adapter, "A binding that assigns a name to a value.")?;
        }
        SyntaxNode::Empty => {
            writeln!(
                adapter,
                "You are editing an empty syntax node. Those get completely \
                ignored at runtime. They exist as placeholders, while you're \
                making up your mind about what you want to type."
            )?;
        }
        SyntaxNode::Expressions { .. } => {
            unreachable!(
                "Expressions node is not directly displayed in the editor."
            );
        }
        SyntaxNode::Function { .. } => {
            writeln!(
                adapter,
                "This is a function literal that produces a function value.",
            )?;
        }
        SyntaxNode::Identifier { .. } => {
            writeln!(adapter, "You are editing an identifier.",)?;
        }
        SyntaxNode::Number { value } => {
            writeln!(
                adapter,
                "This is an integer literal that produces the integer value \
                `{value}`.",
            )?;
        }
        SyntaxNode::Recursion => {
            writeln!(
                adapter,
                "You are editing the `{}` keyword, which calls the current \
                function recursively.",
                node,
            )?;
        }
        SyntaxNode::Tuple { .. } => {
            writeln!(
                adapter,
                "This a tuple literal that produces a tuple value which \
                contains the tuple's children.",
            )?;
        }
    }

    Ok(())
}

struct RenderContext<'r> {
    codebase: &'r Codebase,
    editor: Option<&'r Editor>,
    evaluator: Option<&'r Evaluator>,
    cursor: Option<Cursor>,
}
