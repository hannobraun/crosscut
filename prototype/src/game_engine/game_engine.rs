use crate::{
    io::editor::output::EditorOutput,
    lang::{
        self, editor,
        host::Host,
        interpreter::{StepResult, Value},
    },
};

pub struct GameEngine {
    host: Host,
    lang: lang::Instance,
    game_output: Vec<GameOutput>,
    editor_output: EditorOutput,
}

impl GameEngine {
    pub fn new() -> anyhow::Result<Self> {
        let editor_output = EditorOutput::new()?;

        Ok(Self {
            host: Host::from_functions(["dim"]),
            lang: lang::Instance::new(),
            game_output: Vec::new(),
            editor_output,
        })
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output.render(
            &self.lang.editor,
            &self.lang.code,
            &self.lang.interpreter,
            &self.host,
        )?;

        Ok(())
    }

    pub fn on_editor_input(
        &mut self,
        event: editor::InputEvent,
    ) -> anyhow::Result<()> {
        self.lang.on_event(event, &self.host);

        loop {
            match self.lang.interpreter.step(&self.lang.code) {
                StepResult::CallToHostFunction { id, input, output } => {
                    match id {
                        0 => {
                            // `dim`

                            let Value::Integer { value: input } = input;
                            let Value::Integer { value: output } = output;

                            *output = input / 2;
                        }
                        id => {
                            unreachable!("Undefined host function: `{id}`");
                        }
                    }

                    continue;
                }
                StepResult::CallToIntrinsicFunction => {
                    // Nothing to be done about this.
                    continue;
                }
                StepResult::Error => {
                    // Not handling errors right now. They should be properly
                    // encoded in `Code` and therefore visible in the editor.
                }
                StepResult::Finished { output } => {
                    let Value::Integer { value: output } = output;
                    let color = output as f64 / 255.;

                    self.game_output.push(GameOutput::SubmitColor {
                        color: [color, color, color, 1.],
                    });
                }
            }

            break;
        }

        self.render_editor()?;

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
    }
}

#[derive(Debug)]
pub enum GameInput {
    RenderingFrame,
}

#[derive(Debug, PartialEq)]
pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}
