use crate::lang::{
    code::{Body, Code, Fragment, FragmentKind},
    compiler::compile_and_replace,
    host::Host,
    interpreter::Interpreter,
};

use super::code::Location;

/// # Platform-independent and I/O-less editor core
///
/// ## Implementation Note
///
/// For being platform-independent, the input that this API processes is a bit
/// too specific, dealing with the identity of specific keys.
///
/// For now, this is fine. But eventually, I think we should extract a
/// translation layer. This translation layer can still be platform-independent,
/// but can translate from specific key presses to more higher-level concepts,
/// like "go to parent" or "leave current context", instead of "up" and
/// "escape".
#[derive(Debug)]
pub struct Editor {
    editing: Location,
    input: EditorInput,
}

impl Editor {
    pub fn new(code: &mut Code) -> Self {
        let editing = code.append_to(
            &code.find_innermost_fragment_with_valid_body(),
            Fragment {
                kind: FragmentKind::Empty,
                body: Body::default(),
            },
        );

        Self {
            editing,
            input: EditorInput::new(String::new()),
        }
    }

    pub fn editing(&self) -> &Location {
        &self.editing
    }

    pub fn input(&self) -> &EditorInput {
        &self.input
    }

    pub fn on_input(
        &mut self,
        event: EditorInputEvent,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        match event {
            EditorInputEvent::Character { ch } => {
                if ch.is_whitespace() {
                    self.editing = code.append_to(
                        &code.find_innermost_fragment_with_valid_body(),
                        Fragment {
                            kind: FragmentKind::Empty,
                            body: Body::default(),
                        },
                    );

                    self.input.clear();
                } else {
                    self.input.insert(ch);
                    self.process_code(code, interpreter, host);
                }
            }
            EditorInputEvent::RemoveCharacterLeft => {
                self.input.remove_left();
                self.process_code(code, interpreter, host);
            }
            EditorInputEvent::Left => {
                self.input.move_cursor_left();
            }
            EditorInputEvent::Right => {
                self.input.move_cursor_right();
            }
        }
    }

    fn process_code(
        &mut self,
        code: &mut Code,
        interpreter: &mut Interpreter,
        host: &Host,
    ) {
        self.editing =
            compile_and_replace(&self.input.buffer, &self.editing, host, code);

        if interpreter.state(code).is_running() {
            interpreter.update(code);
        } else {
            interpreter.reset(code);
        }
    }

    pub fn on_command(
        &mut self,
        command: Command,
        code: &mut Code,
        interpreter: &mut Interpreter,
    ) {
        match command {
            Command::Clear => {
                *code = Code::default();
                *self = Self::new(code);
                interpreter.reset(code);
            }
            Command::Nop => {
                // This command does nothing. It exists to give tests something
                // to execute, if they don't want to actually do something
                // except test command interaction itself.
            }
            Command::Reset => {
                interpreter.reset(code);
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct EditorInput {
    buffer: String,
    cursor: usize,
}

impl EditorInput {
    pub fn new(buffer: String) -> Self {
        let cursor = buffer.chars().count();
        Self { buffer, cursor }
    }

    pub fn buffer(&self) -> &String {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.move_cursor_right();
    }

    pub fn remove_left(&mut self) {
        if let Some(cursor) = self.cursor.checked_sub(1) {
            self.buffer.remove(cursor);
            self.move_cursor_left();
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor =
            usize::min(self.cursor.saturating_add(1), self.buffer.len());
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}

#[derive(Debug)]
pub enum EditorInputEvent {
    Character { ch: char },

    RemoveCharacterLeft,
    Left,
    Right,
}

pub enum Command {
    Clear,
    Nop,
    Reset,
}
