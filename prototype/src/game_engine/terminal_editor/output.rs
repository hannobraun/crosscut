use crate::io::terminal_editor::output::EditorOutputAdapter;

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

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.adapter.clear()?;

        self.adapter.flush()?;

        Ok(())
    }
}
