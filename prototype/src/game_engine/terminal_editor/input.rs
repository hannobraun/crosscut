use crate::lang::{
    code::Code,
    editor::{Editor, InputEvent},
    host::Host,
    interpreter::Interpreter,
};

pub struct EditorInput {}

impl EditorInput {
    pub fn on_input(
        &mut self,
        input: InputEvent,
        editor: &mut Editor,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        editor.on_input(input, code, interpreter, host);
    }
}
