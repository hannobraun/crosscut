use super::{code::Code, editor::Editor, interpreter::Interpreter};

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
}
