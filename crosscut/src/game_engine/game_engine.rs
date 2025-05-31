use std::time::{Duration, Instant};

use crate::{
    io::terminal::output::{RawTerminalAdapter, TerminalOutputAdapter},
    language::{
        code::Type,
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
        let mut game_engine = Self {
            game,
            language: Language::new(),
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
            state: State::Running,
        };
        game_engine.run_game_for_a_few_steps();

        game_engine
    }

    pub fn on_editor_input(
        &mut self,
        input: TerminalInput,
    ) -> anyhow::Result<()> {
        let _ = self.game;

        self.editor_input.on_input(input, &mut self.language)?;
        self.run_game_for_a_few_steps();
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

        self.run_game_for_a_few_steps();
        self.render_editor()?;

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
    }

    fn run_game_for_a_few_steps(&mut self) {
        if let State::WaitUntil { instant } = self.state {
            if Instant::now() < instant {
                return;
            }

            match self.language.evaluator().state() {
                RuntimeState::Effect {
                    effect: Effect::ApplyProvidedFunction { name, input: _ },
                    ..
                } => {
                    assert_eq!(
                        name, "sleep_ms",
                        "Expecting to provide output for `sleep_ms` function, \
                        because that is the only one that enters this state.",
                    );

                    self.language
                        .provide_host_function_output(Value::nothing());
                }
                state => {
                    assert!(
                        matches!(state, RuntimeState::Started),
                        "`WaitUntil` state was entered, but expected effect is \
                        not active. This should only happen, if the runtime \
                        has been reset.",
                    );
                }
            }

            self.state = State::Running;
        }

        let mut num_steps = 0;

        loop {
            num_steps += 1;
            if num_steps > 1024 {
                break;
            }

            match self.language.step().clone() {
                RuntimeState::Started | RuntimeState::Running => {
                    continue;
                }
                RuntimeState::Effect { effect, .. } => {
                    match effect {
                        Effect::ApplyProvidedFunction { name, input } => {
                            match name.as_str() {
                                "color" => match input {
                                    Value::Integer { value } => {
                                        let value: f64 = value.into();
                                        let value = value / 255.;

                                        self.game_output.push(
                                            GameOutput::SubmitColor {
                                                color: [
                                                    value, value, value, 1.,
                                                ],
                                            },
                                        );

                                        self.state = State::EndOfFrame;
                                        break;
                                    }
                                    value => {
                                        self.language.trigger_effect(
                                            Effect::UnexpectedInput {
                                                expected: Type::Integer,
                                                actual: value,
                                            },
                                        );
                                    }
                                },
                                "sleep_ms" => match input {
                                    Value::Integer { value } if value >= 0 => {
                                        let value = value as u64;

                                        self.state = State::WaitUntil {
                                            instant: Instant::now()
                                                + Duration::from_millis(value),
                                        };
                                        break;
                                    }
                                    value => {
                                        self.language.trigger_effect(
                                            Effect::UnexpectedInput {
                                                expected: Type::Integer,
                                                actual: value,
                                            },
                                        );
                                    }
                                },
                                _ => {
                                    self.language.trigger_effect(
                                        Effect::ProvidedFunctionNotFound,
                                    );
                                }
                            };
                            continue;
                        }
                        _ => {
                            // We can't handle any other effect.
                            break;
                        }
                    }
                }
                RuntimeState::Finished { output } => {
                    if let Ok(body) = output.into_function_body() {
                        // If the program returns a function, we call that.
                        //
                        // Eventually, we would want something more stringent
                        // here, like expect a `main` function, or a module in a
                        // specific format. For now, this will do though.
                        self.language.apply_function(body);
                        continue;
                    }
                }
            }

            break;
        }
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
