use std::sync::Arc;

use pollster::FutureExt;
use winit::window::Window;

use crate::{
    game_engine::Renderer,
    io::terminal::output::{RawTerminalAdapter, TerminalOutputAdapter},
    language::language::Language,
};

use super::{
    Game, TerminalInput,
    editor::{
        input::{EditorInputOrCommand, TerminalEditorInput},
        output::TerminalEditorOutput,
    },
};

pub struct GameEngine<A> {
    game: Box<dyn Game>,
    language: Language,
    renderer: Renderer,
    editor_input: TerminalEditorInput,
    editor_output: TerminalEditorOutput<A>,
}

impl GameEngine<RawTerminalAdapter> {
    pub fn with_editor_ui(
        game: Box<dyn Game>,
        window: &Arc<Window>,
    ) -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new()?;

        let mut game_engine = Self::new(game, window, adapter)?;
        game_engine.render_editor()?;

        Ok(game_engine)
    }
}

impl<A> GameEngine<A>
where
    A: TerminalOutputAdapter,
{
    pub fn new(
        mut game: Box<dyn Game>,
        window: &Arc<Window>,
        adapter: A,
    ) -> anyhow::Result<Self> {
        let mut language = Language::new();
        let renderer = Renderer::new(window).block_on()?;

        game.on_start(&mut language)?;

        Ok(Self {
            game,
            language,
            renderer,
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

        self.game.on_editor_update(&mut self.language)?;
        self.render_editor()?;

        Ok(())
    }

    pub fn on_frame(&mut self) -> anyhow::Result<()> {
        self.game.on_frame(&mut self.language, &mut self.renderer)?;
        self.render_editor()?;

        Ok(())
    }

    fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output
            .render(&self.language, &self.editor_input)?;

        Ok(())
    }
}
