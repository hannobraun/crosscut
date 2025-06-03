use crate::{
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
    game_output: Vec<GameOutput>,
    editor_input: TerminalEditorInput,
    editor_output: TerminalEditorOutput<A>,
}

impl GameEngine<RawTerminalAdapter> {
    pub fn with_editor_ui(game: Box<dyn Game>) -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new()?;

        let mut game_engine = Self::new(game, adapter)?;
        game_engine.render_editor()?;

        Ok(game_engine)
    }
}

impl<A> GameEngine<A>
where
    A: TerminalOutputAdapter,
{
    pub fn new(mut game: Box<dyn Game>, adapter: A) -> anyhow::Result<Self> {
        let mut language = Language::new();
        let mut game_output = Vec::new();

        game.on_start(&mut language, &mut game_output);

        Ok(Self {
            game,
            language,
            game_output,
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

        self.game
            .on_editor_update(&mut self.language, &mut self.game_output);
        self.render_editor()?;

        Ok(())
    }

    pub fn on_frame(&mut self) -> anyhow::Result<()> {
        self.game
            .on_frame(&mut self.language, &mut self.game_output);
        self.render_editor()?;

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
    }

    fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output
            .render(&self.language, &self.editor_input)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
