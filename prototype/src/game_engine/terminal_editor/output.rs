use crate::{
    io::terminal_editor::output::{Cursor, EditorOutputAdapter},
    language::{code::Codebase, editor::Editor, instance::Language},
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
            codebase: &language.codebase,
            editor: &language.editor,
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
    context.cursor =
        Some(adapter.cursor().move_right(context.editor.input().cursor()));

    if let Some(value) = context.codebase.value {
        write!(adapter, "{value}")?;
    }
    writeln!(adapter)?;

    Ok(())
}

fn render_prompt<A: EditorOutputAdapter>(
    adapter: &mut A,
    editor_input: &TerminalEditorInput,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    match editor_input.mode() {
        EditorMode::Edit => {
            writeln!(
                adapter,
                "Currently editing. Press ESC to enter command mode."
            )?;
        }
        EditorMode::Command { input } => {
            write!(adapter, "> ")?;

            context.cursor = Some(adapter.cursor());

            writeln!(adapter, "{}", input.buffer())?;
            writeln!(
                adapter,
                "Enter command. Press ENTER to confirm, ESC to abort."
            )?;
        }
    }

    Ok(())
}

struct RenderContext<'r> {
    codebase: &'r Codebase,
    editor: &'r Editor,
    cursor: Option<Cursor>,
}
