use std::pin::Pin;

use futures::{Stream, StreamExt};

pub struct Tokenizer {
    chars: Chars,
    buf: String,
}

impl Tokenizer {
    pub fn new(chars: Chars) -> Self {
        Self {
            chars,
            buf: String::new(),
        }
    }

    pub async fn next_token(&mut self) -> Result<Token, TokenizerError> {
        loop {
            let ch = match self.chars.next().await {
                Some(ch) => ch,
                None => {
                    if self.buf.is_empty() {
                        return Err(TokenizerError::NoMoreChars);
                    }

                    return Ok(Token::from_buf(&mut self.buf));
                }
            };

            if ch.is_whitespace() {
                return Ok(Token::from_buf(&mut self.buf));
            }

            self.buf.push(ch);
        }
    }
}

pub type Chars = Pin<Box<dyn Stream<Item = char>>>;

#[derive(Debug)]
pub enum Token {
    CurlyBracketOpen,
    CurlyBracketClose,
    Ident(String),
}

impl Token {
    fn from_buf(buf: &mut String) -> Self {
        let token = match buf.as_str() {
            "{" => Self::CurlyBracketOpen,
            "}" => Self::CurlyBracketClose,
            _ => Self::Ident(buf.clone()),
        };

        buf.clear();

        token
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizerError {
    #[error("No more characters")]
    NoMoreChars,
}
