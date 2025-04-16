use crate::{
    io::editor::output::{
        DebugOutputAdapter, EditorOutputAdapter, RawTerminalAdapter,
    },
    language::{
        code::Type,
        language::Language,
        packages::{Function, Package},
        runtime::{Effect, RuntimeState, Value},
    },
};

use super::{
    TerminalInputEvent,
    editor::{input::TerminalEditorInput, output::TerminalEditorOutput},
};

#[derive(Debug)]
pub struct GameEngine<A> {
    language: Language,
    package: Package<GameEngineFunction>,
    game_output: Vec<GameOutput>,
    editor_input: TerminalEditorInput,
    editor_output: TerminalEditorOutput<A>,
}

impl GameEngine<DebugOutputAdapter> {
    #[cfg(test)]
    pub fn without_editor_ui() -> Self {
        let adapter = DebugOutputAdapter;
        Self::new(adapter)
    }
}

impl GameEngine<RawTerminalAdapter> {
    pub fn with_editor_ui() -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new()?;

        let mut game_engine = Self::new(adapter);
        game_engine.render_editor()?;

        Ok(game_engine)
    }
}

impl<A> GameEngine<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> Self {
        let mut language = Language::new();

        let package = {
            let mut package = language.packages_mut().new_package();

            package.add_function(GameEngineFunction::Color);
            package.add_function(GameEngineFunction::Dim);

            package.build()
        };

        let mut game_engine = Self {
            language,
            package,
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
        };
        game_engine.run_game_for_a_few_steps();

        game_engine
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output
            .render(&self.language, &self.editor_input)?;

        Ok(())
    }

    pub fn on_editor_input(
        &mut self,
        event: TerminalInputEvent,
    ) -> anyhow::Result<()> {
        self.editor_input.on_input(event, &mut self.language);
        self.run_game_for_a_few_steps();
        self.render_editor()?;

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
    }

    fn run_game_for_a_few_steps(&mut self) {
        let mut num_steps = 0;

        loop {
            num_steps += 1;
            if num_steps > 1024 {
                break;
            }

            match self.language.step().clone() {
                RuntimeState::Started | RuntimeState::Running { .. } => {
                    // We're not interested in intermediate values here.
                    continue;
                }
                RuntimeState::Effect { effect, .. } => {
                    match effect {
                        Effect::ProvidedFunction { id, input } => {
                            match self.package.function_by_id(&id) {
                                Some(GameEngineFunction::Color) => {
                                    match input {
                                        Value::Integer { value } => {
                                            self.submit_color(value);
                                            self.language
                                                .provide_host_function_output(
                                                    Value::nothing(),
                                                );
                                        }
                                        value => {
                                            self.language.trigger_effect(
                                                Effect::UnexpectedInput {
                                                    expected: Type::Integer,
                                                    actual: value,
                                                },
                                            );
                                        }
                                    }
                                }
                                Some(GameEngineFunction::Dim) => match input {
                                    Value::Integer { value } => {
                                        self.language
                                            .provide_host_function_output(
                                                Value::Integer {
                                                    value: value / 2,
                                                },
                                            );
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
                                None => {
                                    panic!("Unexpected function: {id:?}");
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
                    // This comment exists to force the current formatting.

                    match output {
                        Value::Integer { value } => {
                            // If the program returns an integer, we use that to
                            // set the color.

                            self.submit_color(value);
                        }
                        value => {
                            match value.into_function_body() {
                                Ok(body) => {
                                    // If the program returns a function, we
                                    // call that function, passing it a display
                                    // value. Using that display value, th
                                    // function can set the color.

                                    self.language
                                        .apply_function(body, Value::nothing());
                                    continue;
                                }
                                Err(_) => {
                                    // The output is neither a number we can use
                                    // for the color, nor a function we can
                                    // call.
                                }
                            }
                        }
                    }
                }
                RuntimeState::Error { .. } => {
                    // Currently not handling errors.
                }
            }

            break;
        }
    }

    fn submit_color(&mut self, value: i32) {
        let value: f64 = value.into();
        let value = value / 255.;

        self.game_output.push(GameOutput::SubmitColor {
            color: [value, value, value, 1.],
        });
    }
}

#[cfg(test)]
impl GameEngine<DebugOutputAdapter> {
    pub fn enter_code(&mut self, code: &str) {
        assert!(self.editor_input.mode().is_edit_mode());

        for ch in code.chars() {
            self.on_char(ch);
        }
    }

    pub fn enter_command_mode(&mut self) {
        self.on_editor_input(TerminalInputEvent::Escape).unwrap();
    }

    pub fn enter_command(&mut self, command: &str) {
        assert!(self.editor_input.mode().is_command_mode());

        for ch in command.chars() {
            self.on_char(ch);
        }
    }

    pub fn execute_command(&mut self) {
        self.on_editor_input(TerminalInputEvent::Enter).unwrap();
    }

    pub fn abort_command(&mut self) {
        self.on_editor_input(TerminalInputEvent::Escape).unwrap();
    }

    pub fn on_char(&mut self, ch: char) {
        self.on_editor_input(TerminalInputEvent::Character { ch })
            .unwrap();
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GameEngineFunction {
    Color,
    Dim,
}

impl Function for GameEngineFunction {
    fn name(&self) -> &str {
        match self {
            Self::Color => "color",
            Self::Dim => "dim",
        }
    }
}
