use std::collections::BTreeSet;

pub struct TerminalEditorInput {}

impl TerminalEditorInput {
    pub fn new() -> Self {
        let mut commands = BTreeSet::new();
        commands.insert("clear");
        commands.insert("nop");
        commands.insert("reset");

        Self {}
    }

    pub fn on_input(&mut self, event: TerminalInputEvent) {
        if let TerminalInputEvent::Character { ch } = event {
            dbg!(ch);
        } else {
            dbg!(event);
        }
    }
}

#[derive(Debug)]
pub enum TerminalInputEvent {
    Character { ch: char },

    Backspace,
    Enter,
    Left,
    Right,
    Escape,
}
