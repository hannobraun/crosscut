use crate::{
    io::editor::output::{EditorOutputAdapter, RawTerminalAdapter},
    language::instance::Language,
};

use super::{
    terminal_editor::{input::TerminalEditorInput, output::EditorOutput},
    TerminalInputEvent,
};

pub struct GameEngine<A> {
    language: Language,
    game_output: Vec<GameOutput>,
    editor_input: TerminalEditorInput,
    editor_output: EditorOutput<A>,
}

impl GameEngine<RawTerminalAdapter> {
    pub fn with_editor_ui() -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new()?;
        Ok(Self::new(adapter))
    }
}

impl<A> GameEngine<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self {
            language: Language::new(),
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output: EditorOutput::new(adapter),
        }
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output.render()?;

        Ok(())
    }

    pub fn on_editor_input(
        &mut self,
        event: TerminalInputEvent,
    ) -> anyhow::Result<()> {
        dbg!(&self.language.codebase);
        self.editor_input.on_input(event, &mut self.language.editor);

        dbg!(&self.language);

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
