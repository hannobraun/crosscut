use super::{
    code::Codebase,
    editor::{Editor, EditorCommand, EditorInputEvent},
    interpreter::Value,
};

#[derive(Debug)]
pub struct Language {
    codebase: Codebase,
    editor: Editor,
}

impl Language {
    pub fn new() -> Self {
        Self {
            codebase: Codebase::new(),
            editor: Editor::new(),
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

    pub fn step(&mut self) -> Value {
        self.codebase.value
    }
}

#[cfg(test)]
impl Language {
    pub fn enter_code(&mut self, code: &str) {
        for ch in code.chars() {
            self.on_input(EditorInputEvent::Insert { ch });
        }
    }
}
