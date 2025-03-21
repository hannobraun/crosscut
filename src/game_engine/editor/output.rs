use crossterm::style::{Attribute, Color};

use crate::{
    io::editor::output::{Cursor, EditorOutputAdapter},
    language::{
        code::{
            Codebase, Expression, IntrinsicFunction, Literal, NodeKind,
            NodePath,
        },
        editor::{Editor, EditorLayout, EditorLine},
        language::Language,
        packages::Packages,
        runtime::{Effect, Evaluator, RuntimeState, Value},
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
    let layout = EditorLayout::new(codebase.root(), codebase.nodes());

    // Creating an empty `Packages` placeholder here, means this function can't
    // be used to render host functions. This is only being used sporadically in
    // tests, so for now, that's good enough.
    //
    // Once it's no longer good enough, that is going to be pretty obvious, I
    // think, because the render code below is going to panic.
    let packages = Packages::new();

    let mut context = RenderContext {
        codebase,
        editor: None,
        evaluator: None,
        packages: &packages,
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
            packages: language.packages(),
            cursor: None,
        };

        let layout = EditorLayout::new(
            context.codebase.root(),
            context.codebase.nodes(),
        );

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
            RuntimeState::Started | RuntimeState::Running { .. } => {
                adapter.color(Color::DarkGreen, |adapter| {
                    writeln!(adapter, "Running")
                })?;
            }
            RuntimeState::Effect { effect, .. } => {
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
            RuntimeState::Error { path } => {
                // While we have a dynamic type system, it's possible that an
                // error is only known at runtime. In that case, we'll get
                // `None` here.
                let maybe_error = context.codebase.errors().get(path);

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
            RuntimeState::Finished { output, .. } => {
                adapter.color(Color::DarkYellow, |adapter| {
                    writeln!(adapter, "Finished: {}", output)
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
        adapter.color(Color::Grey, |adapter| write!(adapter, "·"))?;
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
        if editor.editing() == path {
            context.cursor =
                Some(adapter.cursor().move_right(editor.input().cursor()));
        }
    }

    let color = match node.kind() {
        NodeKind::Expression { expression, .. } => match expression {
            Expression::HostFunction { .. } => Some(Color::DarkMagenta),
            Expression::IntrinsicFunction { .. } => Some(Color::DarkBlue),
        },
        NodeKind::Error { .. } => Some(ERROR_COLOR),
        _ => None,
    };

    let node_display = node.display(context.packages);
    if let Some(color) = color {
        adapter.color(color, |adapter| write!(adapter, "{node_display}"))?;
    } else {
        write!(adapter, "{node_display}")?;
    }

    if let Some(error) = context.codebase.errors().get(path) {
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
    let Some(editor) = context.editor else {
        return Ok(());
    };

    let node = context.codebase.node_at(editor.editing()).node;

    writeln!(adapter)?;

    match node.kind() {
        NodeKind::Empty { .. } => {
            writeln!(
                adapter,
                "You are editing an empty syntax node. Those get completely \
                ignored at runtime. They exist as placeholders, while you're \
                making up your mind about what you want to type."
            )?;
        }
        NodeKind::Expression { expression, .. } => {
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
                Expression::IntrinsicFunction { intrinsic } => {
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

                    match intrinsic {
                        IntrinsicFunction::Drop => {
                            writeln!(
                                adapter,
                                "The `{intrinsic}` function takes any argument \
                                and returns `nothing`.",
                            )?;
                        }
                        IntrinsicFunction::Eval => {
                            writeln!(
                                adapter,
                                "The `{intrinsic}` function expects a function \
                                as an argument and evaluates that function.",
                            )?;
                        }
                        IntrinsicFunction::Identity => {
                            writeln!(
                                adapter,
                                "The `{intrinsic}` function just returns its \
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

                            match literal {
                                Literal::Function => {
                                    writeln!(
                                        adapter,
                                        "This literal returns a function.",
                                    )?;
                                }
                                Literal::Integer { value } => {
                                    writeln!(
                                        adapter,
                                        "This literal returns the integer \
                                        `{value}`.",
                                    )?;
                                }
                                Literal::Tuple => {
                                    writeln!(
                                        adapter,
                                        "This literal returns a tuple.",
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        NodeKind::Recursion { .. } => {
            writeln!(
                adapter,
                "You are editing the `{}` keyword, which calls the current \
                function recursively.",
                node.display(context.packages),
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
    editor: Option<&'r Editor>,
    evaluator: Option<&'r Evaluator>,
    packages: &'r Packages,
    cursor: Option<Cursor>,
}

const ERROR_COLOR: Color = Color::DarkRed;
