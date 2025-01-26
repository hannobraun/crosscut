pub struct GameEngine {}

impl GameEngine {
    pub fn with_editor_ui() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum GameInput {
    RenderingFrame,
}

#[derive(Debug)]
pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
