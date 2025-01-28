use super::{code::Codebase, editor::Editor};

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

    #[cfg(test)]
    pub fn step(&mut self) -> Option<i32> {
        self.codebase.value
    }
}

#[cfg(test)]
impl Language {
    pub fn enter_code(&mut self, code: &str) {
        use super::editor::EditorInputEvent;

        for ch in code.chars() {
            self.editor.on_input(
                EditorInputEvent::Character { ch },
                &mut self.codebase,
            );
        }
    }
}
