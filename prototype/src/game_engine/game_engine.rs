use super::TerminalInputEvent;

pub struct GameEngine {
    game_output: Vec<GameOutput>,
}

impl GameEngine {
    pub fn with_editor_ui() -> anyhow::Result<Self> {
        Ok(Self {
            game_output: Vec::new(),
        })
    }
}

impl GameEngine {
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

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output
            .push(GameOutput::SubmitColor { color: [1.; 4] });
        self.game_output.drain(..)
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
