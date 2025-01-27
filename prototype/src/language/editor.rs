#[derive(Debug)]
pub struct Editor {}

impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn on_input(&mut self, event: EditorInputEvent) {
        if let EditorInputEvent::Character { ch } = event {
            dbg!(ch);
        } else {
            dbg!(event);
        }
    }
}

#[derive(Debug)]
pub enum EditorInputEvent {
    Character { ch: char },
    MoveCursorLeft,
    MoveCursorRight,
    RemoveCharacterLeft,
}