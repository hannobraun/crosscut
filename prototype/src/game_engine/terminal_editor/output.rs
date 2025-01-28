use crate::{
    io::terminal_editor::output::EditorOutputAdapter,
    language::instance::Language,
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

    pub fn render(&mut self, _: &Language) -> anyhow::Result<()> {
        self.adapter.clear()?;

        self.adapter.flush()?;

        Ok(())
    }
}
