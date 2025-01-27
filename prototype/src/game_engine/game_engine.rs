use crate::{
    io::editor::output::{
        DebugOutputAdapter, EditorOutputAdapter, RawTerminalAdapter,
    },
    language::instance::Language,
};

use super::{
    terminal_editor::{
        input::TerminalEditorInput, output::TerminalEditorOutput,
    },
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
        Ok(Self::new(adapter))
    }
}

impl<A> GameEngine<A>
where
    A: EditorOutputAdapter,
{
    pub fn new(adapter: A) -> Self {
        Self {
            language: Language::new(),
            game_output: Vec::new(),
            editor_input: TerminalEditorInput::new(),
            editor_output: TerminalEditorOutput::new(adapter),
        }
    }

    pub fn render_editor(&mut self) -> anyhow::Result<()> {
        self.editor_output.render()?;

        Ok(())
    }

    pub fn on_editor_input(
        &mut self,
        event: TerminalInputEvent,
    ) -> anyhow::Result<()> {
        self.editor_input.on_input(
            event,
            &mut self.language.editor,
            &mut self.language.codebase,
        );

        if let Some(value) = self.language.codebase.value {
            let value: f64 = value.into();
            let value = value / 255.;

            self.game_output.push(GameOutput::SubmitColor {
                color: [value, value, value, 1.],
            });
        }

        Ok(())
    }

    pub fn game_output(&mut self) -> impl Iterator<Item = GameOutput> + '_ {
        self.game_output.drain(..)
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
