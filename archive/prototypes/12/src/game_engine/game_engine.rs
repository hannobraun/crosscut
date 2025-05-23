use crate::{
    io::editor::output::{
        DebugOutputAdapter, EditorOutputAdapter, RawTerminalAdapter,
    },
    language::{
        self,
        host::Host,
        interpreter::{StepResult, Value},
        Language,
    },
};

use super::{
    terminal_editor::{input::TerminalEditorInput, output::EditorOutput},
    TerminalInputEvent,
};

pub struct GameEngine<A> {
    host: Host,
    language: Language,
    game_output: Vec<GameOutput>,
    editor_input: TerminalEditorInput,
    editor_output: EditorOutput<A>,
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
        Ok(Self::new(adapter))
    }
}

impl<A> GameEngine<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> Self {
        let editor_output = EditorOutput::new(adapter);

        Self {
            host: Host::from_functions(["dim"]),
            language: language::Language::new(),
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output,
        }
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output.render(
            &self.editor_input,
            &self.language.editor,
            &self.language.code,
            &self.language.interpreter,
            &self.host,
        )?;

        Ok(())
    }

    pub fn on_editor_input(
        &mut self,
        event: TerminalInputEvent,
    ) -> anyhow::Result<()> {
        self.editor_input.on_input(
            event,
            &mut self.language.editor,
            &mut self.language.code,
            &mut self.language.interpreter,
            &self.host,
        );

        // This is missing some kind of timeout, to prevent an endless loop in
        // Crosscut code from blocking the whole process.
        //
        // As of this writing, it's not yet possible to express endless loops in
        // Crosscut code though, so it's fine.
        loop {
            match self.language.interpreter.step(&self.language.code) {
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
        use crate::game_engine::terminal_editor::input::EditorMode;

        assert!(
            matches!(self.editor_input.mode(), EditorMode::Edit { .. }),
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
        self.on_editor_input(TerminalInputEvent::Character { ch })
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
