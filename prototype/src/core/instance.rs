#[cfg(test)]
use crate::core::editor::{EditorMode, InputEvent};

use super::{code::Code, editor::Editor, interpreter::Interpreter};

#[cfg(test)]
use super::host::Host;

pub struct Instance {
    pub code: Code,
    pub editor: Editor,
    pub interpreter: Interpreter,
}

impl Instance {
    pub fn new() -> Self {
        let code = Code::default();
        let editor = Editor::default();
        let interpreter = Interpreter::new(&code);

        Self {
            code,
            editor,
            interpreter,
        }
    }

    #[cfg(test)]
    pub fn on_command(&mut self, command: &str, host: &Host) {
        assert_eq!(
            *self.editor.mode(),
            EditorMode::Command,
            "Trying to execute command, but editor is not in command mode.",
        );

        for ch in command.chars() {
            self.on_input(InputEvent::Char { value: ch }, host);
        }

        self.on_input(InputEvent::Enter, host);
    }

    #[cfg(test)]
    pub fn on_input(&mut self, event: InputEvent, host: &Host) {
        self.editor.process_input(
            event,
            &mut self.code,
            &mut self.interpreter,
            host,
        );
    }
}
