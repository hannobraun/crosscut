use crate::lang::{
    code::Code,
    editor::{Editor, EditorInputState, InputEvent},
    host::Host,
    interpreter::Interpreter,
};

pub struct EditorInput {
    mode: EditorMode,
}

impl EditorInput {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Edit,
        }
    }

    pub fn mode(&self) -> &EditorMode {
        &self.mode
    }

    pub fn on_input(
        &mut self,
        event: InputEvent,
        editor: &mut Editor,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        match &mut self.mode {
            EditorMode::Command { input } => match event {
                InputEvent::Char { value } => {
                    input.insert(value);
                }
                InputEvent::Backspace => {
                    input.remove_left();
                }
                InputEvent::Enter => {
                    editor.process_command(input, code, interpreter);
                    self.mode = EditorMode::Edit;
                }
                InputEvent::Left => {
                    input.move_cursor_left();
                }
                InputEvent::Right => {
                    input.move_cursor_right();
                }
                InputEvent::Escape => {
                    self.mode = EditorMode::Edit;
                }
            },
            EditorMode::Edit => match event {
                InputEvent::Escape => {
                    self.mode = EditorMode::Command {
                        input: EditorInputState::new(String::new()),
                    };
                }
                event => {
                    editor.on_input(event, code, interpreter, host);
                }
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EditorMode {
    Command { input: EditorInputState },
    Edit,
}

impl EditorMode {
    pub fn is_edit(&self) -> bool {
        matches!(self, Self::Edit)
    }
}
