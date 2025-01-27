use crate::io::editor::output::EditorOutputAdapter;

#[derive(Debug)]
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

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.adapter.clear()?;

        self.adapter.flush()?;

        Ok(())
    }
}
