use super::{code::Codebase, editor::Editor, interpreter::Interpreter};

#[cfg(test)]
use super::{
    editor::{Command, EditorInputEvent},
    host::Host,
    interpreter::{InterpreterState, Value},
};

#[derive(Debug)]
pub struct Language {
    pub code: Codebase,
    pub editor: Editor,
    pub interpreter: Interpreter,
}

impl Language {
    pub fn new() -> Self {
        let mut code = Codebase::default();
        let editor = Editor::new(&mut code);
        let interpreter = Interpreter::new(&code);

        Self {
            code,
            editor,
            interpreter,
        }
    }

    #[cfg(test)]
    pub fn state(&self) -> InterpreterState {
        self.interpreter.state(&self.code)
    }

    #[cfg(test)]
    pub fn on_input(&mut self, input: &str, host: &Host) {
        for ch in input.chars() {
            self.on_char(ch, host);
        }
    }

    #[cfg(test)]
    pub fn on_char(&mut self, ch: char, host: &Host) {
        self.on_event(EditorInputEvent::Character { ch }, host);
    }

    #[cfg(test)]
    pub fn on_event(&mut self, event: EditorInputEvent, host: &Host) {
        self.editor.on_input(
            event,
            &mut self.code,
            &mut self.interpreter,
            host,
        );
    }

    #[cfg(test)]
    pub fn on_command(&mut self, command: Command) {
        self.editor
            .on_command(command, &mut self.code, &mut self.interpreter);
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
