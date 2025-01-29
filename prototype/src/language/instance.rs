use super::{
    code::Codebase,
    editor::{Editor, EditorCommand, EditorInputEvent},
    interpreter::{Interpreter, StepResult},
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
    interpreter: Interpreter,
}

impl Language {
    pub fn new() -> Self {
        let mut codebase = Codebase::new();
        let editor = Editor::new(&mut codebase);

        Self {
            codebase,
            editor,
            interpreter: Interpreter::new(),
        }
    }

    pub fn codebase(&self) -> &Codebase {
        &self.codebase
    }

    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        self.editor.on_input(event, &mut self.codebase);
    }

    pub fn on_command(&mut self, command: EditorCommand) {
        self.editor.on_command(command, &mut self.codebase);
    }

    pub fn step(&mut self) -> StepResult {
        self.interpreter.step(&self.codebase)
    }
}

#[cfg(test)]
use super::interpreter::Value;

#[cfg(test)]
impl Language {
    pub fn enter_code(&mut self, code: &str) {
        for ch in code.chars() {
            let event = if ch.is_whitespace() {
                EditorInputEvent::SubmitToken
            } else {
                EditorInputEvent::Insert { ch }
            };

            self.on_input(event);
        }
    }

    pub fn step_until_finished(&mut self) -> Value {
        match self.step() {
            StepResult::Finished { output } => output,
        }
    }
}
