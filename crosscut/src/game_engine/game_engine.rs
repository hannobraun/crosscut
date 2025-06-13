use std::sync::Arc;

use pollster::FutureExt;
use winit::{keyboard::KeyCode, window::Window};

use crate::{
    language::language::Language,
    terminal::{RawTerminalAdapter, TerminalOutputAdapter},
};

use super::{
    Game, Init, TerminalInput,
    editor::{
        input::{EditorInputOrCommand, TerminalEditorInput},
        output::TerminalEditorOutput,
    },
};

pub struct GameEngine<A> {
    game: Box<dyn Game>,
    language: Language,
    editor_input: TerminalEditorInput,
    editor_output: TerminalEditorOutput<A>,
}

impl GameEngine<RawTerminalAdapter> {
    pub fn with_editor_ui(
        init: Box<dyn Init>,
        window: &Arc<Window>,
    ) -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new()?;

        let mut game_engine = Self::new(init, window, adapter)?;
        game_engine.render_editor()?;

        Ok(game_engine)
    }
}

impl<A> GameEngine<A>
where
    A: TerminalOutputAdapter,
{
    pub fn new(
        mut init: Box<dyn Init>,
        window: &Arc<Window>,
        adapter: A,
    ) -> anyhow::Result<Self> {
        let mut language = Language::new();

        let mut game = init.init(&mut language, window).block_on()?;
        game.on_code_update(&mut language)?;

        Ok(Self {
            game,
            language,
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
        })
    }

    pub fn on_terminal_input(
        &mut self,
        input: TerminalInput,
    ) -> anyhow::Result<()> {
        match self.editor_input.on_input(input) {
            Some(EditorInputOrCommand::Input { input }) => {
                self.language.on_editor_input(input);
            }
            Some(EditorInputOrCommand::Command { command }) => {
                self.language.on_editor_command(command)?;
            }
            None => {}
        }

        self.game.on_code_update(&mut self.language)?;
        self.render_editor()?;

        Ok(())
    }

    pub fn on_window_resized(&mut self, new_size: [u32; 2]) {
        self.game.on_window_resized(new_size);
    }

    pub fn on_key(&mut self, key: KeyCode) {
        self.game.on_key(key);
    }

    pub fn on_frame(&mut self) -> anyhow::Result<()> {
        self.game.on_frame(&mut self.language)?;
        self.render_editor()?;

        Ok(())
    }

    fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output
            .render(&self.language, &self.editor_input)?;

        Ok(())
    }
}
