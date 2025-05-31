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

        let mut game_engine = Self::new(game, adapter);
        game_engine.render_editor()?;

        Ok(game_engine)
    }
}

impl<A> GameEngine<A>
where
    A: TerminalOutputAdapter,
{
    pub fn new(mut game: Box<dyn Game>, adapter: A) -> Self {
        let mut language = Language::new();
        let mut game_output = Vec::new();

        game.on_start(&mut language, &mut game_output);

        Self {
            game,
            language,
            game_output,
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
        }
    }

    pub fn on_terminal_input(
        &mut self,
        input: TerminalInput,
    ) -> anyhow::Result<()> {
        match self.editor_input.on_input(input) {
            Some(EditorInputOrCommand::Input { input }) => {
                self.language.on_input(input);
            }
            Some(EditorInputOrCommand::Command { command }) => {
                self.language.on_command(command)?;
            }
            None => {}
        }
        self.game
            .on_editor_input(&mut self.language, &mut self.game_output);
        self.render_editor()?;

        Ok(())
    }

    pub fn on_frame(&mut self) -> anyhow::Result<()> {
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

#[cfg(test)]
use crate::io::terminal::output::DebugOutputAdapter;

#[cfg(test)]
impl GameEngine<DebugOutputAdapter> {
    pub fn without_editor_ui() -> Self {
        let game = Box::new(super::PureCrosscutGame::default());
        let adapter = DebugOutputAdapter;
        Self::new(game, adapter)
    }

    pub fn enter_code(&mut self, code: &str) -> &mut Self {
        assert!(self.editor_input.mode().is_edit_mode());

        for ch in code.chars() {
            self.on_char(ch);
        }

        self
    }

    pub fn cursor_down(&mut self) -> &mut Self {
        assert!(self.editor_input.mode().is_edit_mode());
        self.on_terminal_input(TerminalInput::Down).unwrap();
        self
    }

    pub fn enter_command_mode(&mut self) -> &mut Self {
        self.on_terminal_input(TerminalInput::Escape).unwrap();
        self
    }

    pub fn enter_command(&mut self, command: &str) -> &mut Self {
        assert!(self.editor_input.mode().is_command_mode());

        for ch in command.chars() {
            self.on_char(ch);
        }

        self
    }

    pub fn execute_command(&mut self) -> &mut Self {
        self.on_terminal_input(TerminalInput::Enter).unwrap();
        self
    }

    pub fn abort_command(&mut self) -> &mut Self {
        self.on_terminal_input(TerminalInput::Escape).unwrap();
        self
    }

    pub fn on_char(&mut self, ch: char) -> &mut Self {
        self.on_terminal_input(TerminalInput::Character { ch })
            .unwrap();
        self
    }
}

#[derive(Debug)]
pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
