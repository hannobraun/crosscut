use crate::{
    io::editor::output::{
        EditorOutput, EditorOutputAdapter, RawTerminalAdapter,
    },
    lang::{
        self, editor,
        host::Host,
        interpreter::{StepResult, Value},
    },
};

pub struct GameEngine<A> {
    host: Host,
    lang: lang::Instance,
    game_output: Vec<GameOutput>,
    editor_output: EditorOutput<A>,
}

impl GameEngine<RawTerminalAdapter> {
    pub fn with_editor() -> anyhow::Result<Self> {
        let adapter = RawTerminalAdapter::new();
        Self::new(adapter)
    }
}

impl<A> GameEngine<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> anyhow::Result<Self> {
        let editor_output = EditorOutput::new(adapter)?;

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

    #[cfg(test)]
    pub fn on_code(&mut self, code: &str) {
        use crate::lang::editor::EditorMode;

        assert!(
            matches!(self.lang.editor.mode(), EditorMode::Edit { .. }),
            "Trying to input code, but editor is not in edit mode.",
        );

        self.on_input(code);
    }

    #[cfg(test)]
    pub fn on_input(&mut self, input: &str) {
        for ch in input.chars() {
            self.on_char(ch);
        }
    }

    #[cfg(test)]
    pub fn on_char(&mut self, ch: char) {
        self.on_editor_input(editor::InputEvent::Char { value: ch })
            .unwrap();
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
