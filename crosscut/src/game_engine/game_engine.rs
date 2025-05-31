use crate::{
    io::terminal::output::{RawTerminalAdapter, TerminalOutputAdapter},
    language::{
        language::Language,
        runtime::{Effect, RuntimeState, Value},
    },
};

use super::{
    Game, TerminalInput,
    editor::{input::TerminalEditorInput, output::TerminalEditorOutput},
    game::State,
};

pub struct GameEngine<A> {
    game: Box<dyn Game>,
    language: Language,
    game_output: Vec<GameOutput>,
    editor_input: TerminalEditorInput,
    editor_output: TerminalEditorOutput<A>,
    state: State,
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
    pub fn new(game: Box<dyn Game>, adapter: A) -> Self {
        let language = Language::new();
        let game_output = Vec::new();
        let state = State::Running;

        let mut game_engine = Self {
            game,
            language,
            game_output,
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
            state,
        };
        game_engine.game.run_game_for_a_few_steps(
            &mut game_engine.state,
            &mut game_engine.language,
            &mut game_engine.game_output,
        );

        game_engine
    }

    pub fn on_editor_input(
        &mut self,
        input: TerminalInput,
    ) -> anyhow::Result<()> {
        self.editor_input.on_input(input, &mut self.language)?;
        self.game.run_game_for_a_few_steps(
            &mut self.state,
            &mut self.language,
            &mut self.game_output,
        );
        self.render_editor()?;

        Ok(())
    }

    pub fn on_frame(&mut self) -> anyhow::Result<()> {
        if let State::EndOfFrame = self.state {
            match self.language.evaluator().state() {
                RuntimeState::Effect {
                    effect: Effect::ApplyProvidedFunction { name, input: _ },
                    ..
                } => {
                    assert_eq!(
                        name, "color",
                        "Expecting to provide output for `color` function, \
                        because that is the only one that enters this state.",
                    );

                    self.language
                        .provide_host_function_output(Value::nothing());
                }
                state => {
                    assert!(
                        matches!(state, RuntimeState::Started),
                        "`EndOfFrame` state was entered, but expected effect \
                        is not active. This should only happen, if the runtime \
                        has been reset.",
                    );
                }
            }

            self.state = State::Running;
        }

        self.game.run_game_for_a_few_steps(
            &mut self.state,
            &mut self.language,
            &mut self.game_output,
        );
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
        let game = Box::new(super::PureCrosscutGame);
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
        self.on_editor_input(TerminalInput::Down).unwrap();
        self
    }

    pub fn enter_command_mode(&mut self) -> &mut Self {
        self.on_editor_input(TerminalInput::Escape).unwrap();
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
        self.on_editor_input(TerminalInput::Enter).unwrap();
        self
    }

    pub fn abort_command(&mut self) -> &mut Self {
        self.on_editor_input(TerminalInput::Escape).unwrap();
        self
    }

    pub fn on_char(&mut self, ch: char) -> &mut Self {
        self.on_editor_input(TerminalInput::Character { ch })
            .unwrap();
        self
    }
}

#[derive(Debug)]
pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
