use super::{
    code::Codebase,
    editor::{Editor, EditorCommand, EditorInputEvent},
    host::Host,
    interpreter::{Interpreter, StepResult},
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
    interpreter: Interpreter,
    host: Host,
}

impl Language {
    pub fn with_host(host: Host) -> Self {
        let mut codebase = Codebase::new();
        let editor = Editor::new(&mut codebase);
        let interpreter = Interpreter::new(&codebase);

        Self {
            codebase,
            editor,
            interpreter,
            host,
        }
    }

    pub fn codebase(&self) -> &Codebase {
        &self.codebase
    }

    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        self.editor.on_input(
            event,
            &mut self.codebase,
            &mut self.interpreter,
            &self.host,
        );
    }

    pub fn on_command(&mut self, command: EditorCommand) {
        self.editor.on_command(
            command,
            &mut self.codebase,
            &mut self.interpreter,
        );
    }

    pub fn step(&mut self) -> StepResult {
        self.interpreter.step(&self.codebase)
    }
}

#[cfg(test)]
use super::interpreter::Value;

#[cfg(test)]
impl Language {
    pub fn without_host() -> Self {
        Self::with_host(Host::new())
    }

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
        let mut i = 0;

        loop {
            match self.step() {
                StepResult::Application { output: _ } => {
                    // We're not concerned with intermediate results here.
                }
                StepResult::Finished { output } => {
                    break output;
                }
                StepResult::Error => {
                    panic!("Unexpected runtime error.");
                }
            }

            i += 1;

            if i > 1024 {
                // This function is only used in tests. And those are not so
                // complicated, as to require a large number of steps.
                panic!("Test seemingly ran into an endless loop.");
            }
        }
    }
}
