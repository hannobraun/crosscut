use std::pin::Pin;

use futures::{Stream, StreamExt};

pub struct Tokenizer<'r> {
    chars: Pin<&'r mut dyn Stream<Item = char>>,
    buf: String,
}

impl<'r> Tokenizer<'r> {
    pub fn new(chars: Pin<&'r mut dyn Stream<Item = char>>) -> Self {
        Self {
            chars,
            buf: String::new(),
        }
    }

    pub async fn next_token(&mut self) -> Option<String> {
        loop {
            let ch = match self.chars.next().await {
                Some(ch) => ch,
                None => {
                    if self.buf.is_empty() {
                        return None;
                    }

                    let token = self.buf.clone();
                    self.buf.clear();
                    return Some(token);
                }
            };

            if ch.is_whitespace() {
                let token = self.buf.clone();
                self.buf.clear();
                return Some(token);
            }

            self.buf.push(ch);
        }
    }
}
