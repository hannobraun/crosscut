use super::{
    code::Codebase,
    editor::{Editor, EditorInputEvent},
};

#[derive(Debug)]
pub struct Language {
    pub codebase: Codebase,
    pub editor: Editor,
}

impl Language {
    pub fn new() -> Self {
        Self {
            codebase: Codebase::new(),
            editor: Editor::new(),
        }
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        self.editor.on_input(event, &mut self.codebase);
    }

    pub fn step(&mut self) -> Option<i32> {
        self.codebase.value
    }
}

#[cfg(test)]
impl Language {
    pub fn enter_code(&mut self, code: &str) {
        use super::editor::EditorInputEvent;

        for ch in code.chars() {
            self.on_input(EditorInputEvent::Character { ch });
        }
    }
}
