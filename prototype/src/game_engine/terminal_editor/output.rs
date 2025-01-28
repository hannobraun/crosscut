use crate::{
    io::terminal_editor::output::EditorOutputAdapter,
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
        render_prompt(&mut self.adapter, editor_input)?;

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
    context.cursor = {
        let [x, y] = adapter.cursor();
        let x = {
            let x: usize = x.into();
            let x = x.saturating_add(context.editor.input().cursor());
            let x: u16 = x.try_into().unwrap_or(u16::MAX);
            x
        };

        Some([x, y])
    };

    if let Some(value) = context.codebase.value {
        write!(adapter, "{value}")?;
    }
    writeln!(adapter)?;

    Ok(())
}

fn render_prompt<A: EditorOutputAdapter>(
    adapter: &mut A,
    editor_input: &TerminalEditorInput,
) -> anyhow::Result<()> {
    match editor_input.mode() {
        EditorMode::Edit => {
            writeln!(
                adapter,
                "Currently editing. Press ESC to enter command mode."
            )?;
        }
        EditorMode::Command => {
            writeln!(adapter, "> ")?;
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
    cursor: Option<[u16; 2]>,
}
