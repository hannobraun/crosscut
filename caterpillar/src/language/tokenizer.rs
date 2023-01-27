pub struct Tokenizer<'r, Chars: Iterator<Item = char> + 'r> {
    pub chars: Chars,
    pub buf: &'r mut Buf,
}

impl<'r, Chars: Iterator<Item = char> + 'r> Iterator for Tokenizer<'r, Chars> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state = State::NotStarted;

        for ch in &mut self.chars {
            match state {
                State::NotStarted => {
                    if ch == '[' {
                        return Some(Token::ArrayOpen);
                    }
                    if ch == ']' {
                        return Some(Token::ArrayClose);
                    }

                    if !ch.is_whitespace() {
                        state = State::ReadingFn;
                        self.buf.push(ch);
                    }
                }
                State::ReadingFn => {
                    if ch.is_whitespace() {
                        break;
                    }

                    self.buf.push(ch);
                }
            }
        }

        if self.buf.is_empty() {
            return None;
        }

        let token = Token::Fn {
            name: self.buf.clone(),
        };
        self.buf.clear();

        Some(token)
    }
}

pub type Buf = String;

pub enum Token {
    /// Refers to a function
    Fn { name: String },

    /// Opens an array
    ArrayOpen,

    /// Closes an array
    ArrayClose,
}

enum State {
    NotStarted,
    ReadingFn,
}
