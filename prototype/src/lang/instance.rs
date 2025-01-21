#[cfg(test)]
use crate::lang::editor::EditorMode;

use super::{
    code::Code,
    editor::{Editor, InputEvent},
    host::Host,
    interpreter::Interpreter,
};

#[cfg(test)]
use super::interpreter::Value;

#[derive(Debug)]
pub struct Instance {
    pub code: Code,
    pub editor: Editor,
    pub interpreter: Interpreter,
}

impl Instance {
    pub fn new() -> Self {
        let mut code = Code::default();
        let editor = Editor::new(&mut code);
        let interpreter = Interpreter::new(&code);

        Self {
            code,
            editor,
            interpreter,
        }
    }

    #[cfg(test)]
    pub fn on_code(&mut self, code: &str, host: &Host) {
        assert!(
            matches!(self.editor.mode(), EditorMode::Edit { .. }),
            "Trying to input code, but editor is not in edit mode.",
        );

        self.on_input(code, host);
    }

    #[cfg(test)]
    pub fn on_input(&mut self, input: &str, host: &Host) {
        for ch in input.chars() {
            self.on_char(ch, host);
        }
    }

    #[cfg(test)]
    pub fn on_char(&mut self, ch: char, host: &Host) {
        self.on_event(InputEvent::Char { value: ch }, host);
    }

    pub fn on_event(&mut self, event: InputEvent, host: &Host) {
        self.editor.process_input(
            event,
            &mut self.code,
            &mut self.interpreter,
            host,
        );
    }

    #[cfg(test)]
    pub fn run_until_finished(&mut self) -> Value {
        use super::interpreter::StepResult;

        loop {
            match self.interpreter.step(&self.code) {
                StepResult::CallToHostFunction { id, .. } => {
                    panic!("Unexpected call to host function `{id}`");
                }
                StepResult::CallToIntrinsicFunction => {
                    // No need to do anything about this.
                }
                StepResult::Error => {
                    panic!("Unexpected error while stepping interpreter.");
                }
                StepResult::Finished { output } => {
                    break output;
                }
            }
        }
    }
}
