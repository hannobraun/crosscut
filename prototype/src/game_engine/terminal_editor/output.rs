use crate::{
    io::terminal_editor::output::EditorOutputAdapter,
    language::{code::Codebase, instance::Language},
};

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

    pub fn render(&mut self, language: &Language) -> anyhow::Result<()> {
        let mut context = RenderContext {
            codebase: &language.codebase,
        };

        self.adapter.clear()?;

        render_code(&mut self.adapter, &mut context)?;

        self.adapter.flush()?;

        Ok(())
    }
}

fn render_code<A: EditorOutputAdapter>(
    adapter: &mut A,
    context: &mut RenderContext,
) -> anyhow::Result<()> {
    if let Some(value) = context.codebase.value {
        write!(adapter, "{value}")?;
    }

    Ok(())
}

struct RenderContext<'r> {
    codebase: &'r Codebase,
}
