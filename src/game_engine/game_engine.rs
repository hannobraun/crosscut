use crate::{
    io::editor::output::{
        DebugOutputAdapter, EditorOutputAdapter, RawTerminalAdapter,
    },
    language::{
        code::Type,
        instance::Language,
        packages::{Function, FunctionId, Package},
        runtime::{Effect, StepResult, Value, ValueWithSource},
    },
};

use super::{
    editor::{input::TerminalEditorInput, output::TerminalEditorOutput},
    TerminalInputEvent,
};

#[derive(Debug)]
pub struct GameEngine<A> {
    language: Language,
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
        let mut package = Package::new();
        package.function(GameEngineFunction::Dim);

        let mut game_engine = Self {
            language: Language::with_package(package),
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
        };
        game_engine.run_game_until_finished();

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
        self.run_game_until_finished();
        self.render_editor()?;

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
    }

    fn run_game_until_finished(&mut self) {
        loop {
            match self.language.step() {
                StepResult::FunctionApplied { output: _ } => {
                    // We're not interested in intermediate values here.
                    continue;
                }
                StepResult::EffectTriggered { effect } => match effect {
                    Effect::ApplyHostFunction { id, input } => {
                        self.apply_host_function(id, input);
                        continue;
                    }
                    _ => {
                        // We can't handle any other effect.
                        break;
                    }
                },
                StepResult::Finished {
                    output:
                        ValueWithSource {
                            inner: Value::Integer { value },
                            ..
                        },
                } => {
                    let value: f64 = value.into();
                    let value = value / 255.;

                    self.game_output.push(GameOutput::SubmitColor {
                        color: [value, value, value, 1.],
                    });
                }
                StepResult::Finished { output: _ } => {
                    // The output is not a number. We can't render that.
                }
                StepResult::Error => {
                    // Currently not handling errors.
                }
            }

            break;
        }
    }

    fn apply_host_function(&mut self, id: FunctionId, input: Value) {
        match Function::from_verified_id(id) {
            GameEngineFunction::Dim => match input {
                Value::Integer { value } => {
                    self.language.provide_host_function_output(
                        Value::Integer { value: value / 2 },
                    );
                }
                value => {
                    self.language.trigger_effect(Effect::UnexpectedInput {
                        expected: Type::Integer,
                        actual: value,
                    });
                }
            },
        }
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

pub enum GameEngineFunction {
    Dim,
}

impl Function for GameEngineFunction {
    fn from_id(FunctionId { id }: FunctionId) -> Option<Self> {
        match id {
            0 => Some(Self::Dim),
            _ => None,
        }
    }

    fn id(&self) -> FunctionId {
        let id = match self {
            Self::Dim => 0,
        };

        FunctionId { id }
    }

    fn name(&self) -> &str {
        match self {
            Self::Dim => "dim",
        }
    }
}
