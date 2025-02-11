use crossterm::style::{Attribute, Color};

use crate::{
    io::editor::output::{Cursor, EditorOutputAdapter},
    language::{
        code::{
            Codebase, Expression, IntrinsicFunction, LocatedNode, NodeKind,
        },
        editor::Editor,
        instance::Language,
        packages::Package,
        runtime::{Effect, Evaluator, EvaluatorState, Value},
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
            evaluator: language.evaluator(),
            package: language.package(),
            cursor: None,
        };

        self.adapter.clear()?;

        render_interpreter_state(&mut self.adapter, &context)?;
        render_code(&mut self.adapter, &mut context)?;
        render_prompt(&mut self.adapter, editor_input, &mut context)?;
        render_help(&mut self.adapter, &context)?;

        if let Some(Cursor { inner: [x, y] }) = context.cursor {
            self.adapter.move_cursor_to(x, y)?;
        }

        self.adapter.flush()?;

        Ok(())
    }
}

fn render_interpreter_state<A: EditorOutputAdapter>(
    adapter: &mut A,
    context: &RenderContext,
) -> anyhow::Result<()> {
    adapter.attribute(Attribute::Bold, |adapter| {
        match context.evaluator.state(context.codebase) {
            EvaluatorState::Running { .. }
            | EvaluatorState::IgnoringEmptyFragment { .. }
            | EvaluatorState::Recursing => {
                adapter.color(Color::DarkGreen, |adapter| {
                    writeln!(adapter, "Running")
                })?;
            }
            EvaluatorState::Effect { effect, .. } => {
                adapter.color(Color::DarkCyan, |adapter| {
                    write!(adapter, "Effect: ")?;

                    match effect {
                        Effect::ApplyHostFunction { id, input } => {
                            writeln!(
                                adapter,
                                "apply host function `{id:?}` (input: {input})",
                            )?;
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
            EvaluatorState::Error { path } => {
                // While we have a dynamic type system, it's possible that an
                // error is only known at runtime. In that case, we'll get
                // `None` here.
                let maybe_error = context.codebase.error_at(&path);

                adapter.color(ERROR_COLOR, |adapter| {
                    write!(adapter, "Error")?;

                    if let Some(error) = maybe_error {
                        writeln!(adapter, ": {error}")?;
                    } else {
                        writeln!(adapter)?;
                    }

                    Ok(())
                })?;
            }
            EvaluatorState::Finished { output } => {
                adapter.color(Color::DarkYellow, |adapter| {
                    writeln!(adapter, "Finished: {}", output.inner)
                })?;
            }
        }

        Ok(())
    })?;

    Ok(())
}

fn render_code<A: EditorOutputAdapter>(
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    writeln!(adapter)?;

    for located_node in context.codebase.leaf_to_root() {
        render_possibly_active_node(located_node, adapter, context)?;
    }

    writeln!(adapter)?;

    Ok(())
}

fn render_possibly_active_node<A: EditorOutputAdapter>(
    located_node: LocatedNode,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    let is_active_node = context.evaluator.state(context.codebase).path()
        == Some(&located_node.path);

    if is_active_node {
        adapter.attribute(Attribute::Bold, |adapter| {
            write!(adapter, " => ")?;
            render_node(located_node, adapter, context)
        })?;
    } else {
        write!(adapter, "    ")?;
        render_node(located_node, adapter, context)?;
    }

    writeln!(adapter)?;

    Ok(())
}

fn render_node<A: EditorOutputAdapter>(
    located_node: LocatedNode,
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    if context.editor.editing() == &located_node.path {
        context.cursor =
            Some(adapter.cursor().move_right(context.editor.input().cursor()));
    }

    let color = match &located_node.node.kind {
        NodeKind::Expression { expression } => match expression {
            Expression::HostFunction { .. } => Some(Color::DarkMagenta),
            Expression::IntrinsicFunction { .. } => Some(Color::DarkBlue),
        },
        NodeKind::Error { .. } => Some(ERROR_COLOR),
        _ => None,
    };

    let node_display = located_node.node.display(context.package);
    if let Some(color) = color {
        adapter.color(color, |adapter| write!(adapter, "{node_display}"))?;
    } else {
        write!(adapter, "{node_display}")?;
    }

    if let Some(error) = context.codebase.error_at(&located_node.path) {
        adapter.color(ERROR_COLOR, |adapter| {
            write!(adapter, "    error: {error}")
        })?;
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
    let path = context.editor.editing();
    let node = context.codebase.node_at(path);

    writeln!(adapter)?;

    match &node.kind {
        NodeKind::Empty => {
            writeln!(
                adapter,
                "You are editing an empty syntax node. Those get completely \
                ignored at runtime. They exist as placeholders, while you're \
                making up your mind about what you want to type."
            )?;
        }
        NodeKind::Expression { expression } => {
            write!(adapter, "You are editing an expression. ")?;

            match expression {
                Expression::HostFunction { id: _ } => {
                    writeln!(
                        adapter,
                        "This expression is the application of a host \
                        function. Those are defined outside of Crosscut, by \
                        the host. They allow your program to interact with the \
                        outside world (in whatever way the host allows you to \
                        do so).",
                    )?;
                }
                Expression::IntrinsicFunction { function } => {
                    writeln!(
                        adapter,
                        "This expression is the application of an intrinsic \
                        function. Intrinsic functions are built into Crosscut, \
                        and are available to every Crosscut program.",
                    )?;
                    writeln!(adapter)?;
                    writeln!(
                        adapter,
                        "Intrinsic functions never allow interaction with the \
                        outside world though (except maybe in some limited \
                        ways with the development environment, to help with \
                        debugging). To do that, you need to call a host \
                        function.",
                    )?;

                    writeln!(adapter)?;

                    match function {
                        IntrinsicFunction::Identity => {
                            writeln!(
                                adapter,
                                "The `{function}` function just returns its \
                                input unchanged.",
                            )?;
                        }
                        IntrinsicFunction::Literal { literal } => {
                            writeln!(
                                adapter,
                                "This is a special kind of intrinsic function, \
                                a literal. Literals are functions that take \
                                `{}` and return the value they represent.",
                                Value::Nothing,
                            )?;

                            writeln!(adapter)?;

                            let value = literal.to_value(path);
                            match value {
                                Value::Nothing => {
                                    writeln!(
                                        adapter,
                                        "This literal returns the `{value}` \
                                        value.",
                                    )?;
                                }
                                Value::Function { hash: _ } => {
                                    writeln!(
                                        adapter,
                                        "This literal returns a function.",
                                    )?;
                                }
                                Value::Integer { value } => {
                                    writeln!(
                                        adapter,
                                        "This literal returns the integer \
                                        `{value}`.",
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        NodeKind::Recursion => {
            writeln!(
                adapter,
                "You are editing the `{}` keyword, which calls the current \
                function recursively.",
                node.display(context.package),
            )?;
        }
        NodeKind::Error { .. } => {
            writeln!(adapter, "You are editing an erroneous syntax node.",)?;
        }
    }

    Ok(())
}

struct RenderContext<'r> {
    codebase: &'r Codebase,
    editor: &'r Editor,
    evaluator: &'r Evaluator,
    package: &'r Package,
    cursor: Option<Cursor>,
}

const ERROR_COLOR: Color = Color::DarkRed;
