use super::code::Codebase;

#[derive(Debug)]
pub struct Editor {
    input: EditorInput,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            input: EditorInput::new(String::new()),
        }
    }

    pub fn on_input(&mut self, event: EditorInputEvent, _: &mut Codebase) {
        if let EditorInputEvent::Character { ch } = event {
            self.input.insert(ch);
        } else {
            dbg!(event);
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct EditorInput {
    buffer: String,
}

impl EditorInput {
    pub fn new(buffer: String) -> Self {
        Self { buffer }
    }

    pub fn insert(&mut self, ch: char) {
        self.buffer.push(ch);
    }
}

#[derive(Debug)]
pub enum EditorInputEvent {
    Character { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    RemoveCharacterLeft,
}
