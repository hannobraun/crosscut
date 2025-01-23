use crate::lang::{
    code::Code,
    editor::{Editor, InputEvent},
    host::Host,
    interpreter::Interpreter,
};

pub struct EditorInput {}

impl EditorInput {
    pub fn new() -> Self {
        Self {}
    }

    pub fn on_input(
        &mut self,
        event: InputEvent,
        editor: &mut Editor,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        editor.on_input(event, code, interpreter, host);
    }
}
