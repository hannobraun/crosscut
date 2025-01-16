use super::{code::Code, editor::Editor, interpreter::Interpreter};

pub struct Instance {
    pub code: Code,
    pub editor: Editor,
    pub interpreter: Interpreter,
}
