use super::TerminalInputEvent;

pub struct GameEngine {}

impl GameEngine {
    pub fn with_editor_ui() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn on_editor_input(
        &mut self,
        event: TerminalInputEvent,
    ) -> anyhow::Result<()> {
        if let TerminalInputEvent::Character { ch } = event {
            dbg!(ch);
        }

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
