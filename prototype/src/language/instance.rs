use super::editor::Editor;

#[derive(Debug)]
pub struct Language {
    pub editor: Editor,
}

impl Language {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
        }
    }
}
