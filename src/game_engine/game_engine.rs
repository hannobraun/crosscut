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
    end_of_frame: bool,
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

        let package = language
            .packages_mut()
            .new_package([GameEngineFunction::Color, GameEngineFunction::Dim]);

        let mut game_engine = Self {
            language,
            package,
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
            end_of_frame: false,
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

    pub fn on_frame(&mut self) -> anyhow::Result<()> {
        self.run_game_for_a_few_steps();
        self.render_editor()?;

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
    }

    fn run_game_for_a_few_steps(&mut self) {
        if self.end_of_frame {
            match self.language.evaluator().state() {
                RuntimeState::Effect {
                    effect: Effect::ProvidedFunction { id, .. },
                    ..
                } => {
                    assert_eq!(
                        self.package.function_by_id(id),
                        Some(&GameEngineFunction::Color),
                        "Expecting to provide output for `color` function, \
                        because that is the only one that sets the \
                        `end_of_frame` flag.",
                    );

                    self.language
                        .provide_host_function_output(Value::nothing());
                }
                state => {
                    assert!(
                        matches!(state, RuntimeState::Started),
                        "`end_of_frame` flag has been set, but expected effect \
                        is not active. This should only happen, if the runtime \
                        has been reset.",
                    );
                }
            }

            self.end_of_frame = false;
        }

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
                                            self.end_of_frame = true;
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
    pub fn enter_code(&mut self, code: &str) -> &mut Self {
        assert!(self.editor_input.mode().is_edit_mode());

        for ch in code.chars() {
            self.on_char(ch);
        }

        self
    }

    pub fn cursor_down(&mut self) -> &mut Self {
        assert!(self.editor_input.mode().is_edit_mode());
        self.on_editor_input(TerminalInputEvent::Down).unwrap();
        self
    }

    pub fn enter_command_mode(&mut self) -> &mut Self {
        self.on_editor_input(TerminalInputEvent::Escape).unwrap();
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
        self.on_editor_input(TerminalInputEvent::Enter).unwrap();
        self
    }

    pub fn abort_command(&mut self) -> &mut Self {
        self.on_editor_input(TerminalInputEvent::Escape).unwrap();
        self
    }

    pub fn on_char(&mut self, ch: char) -> &mut Self {
        self.on_editor_input(TerminalInputEvent::Character { ch })
            .unwrap();
        self
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
