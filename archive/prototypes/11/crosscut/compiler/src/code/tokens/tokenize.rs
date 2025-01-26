use std::mem;

use crate::code::tokens::Token;

use super::{Keyword::*, Punctuator::*};

pub fn tokenize(input: &str) -> Vec<Token> {
    let eager_tokens = vec![
        (r",", Token::Punctuator(Delimiter)),
        (r":", Token::Punctuator(Introducer)),
        (r"->", Token::Punctuator(Transformer)),
        (r".", Token::Punctuator(Terminator)),
    ];

    let mut state = State::Initial;
    let mut buffer = Buffer::default();

    let mut tokens = Vec::new();

    for ch in input.chars() {
        match state {
            State::Initial => match ch {
                '#' => {
                    buffer.take_literal_or_keyword_or_identifier(&mut tokens);
                    state = State::Comment;
                }
                ch if ch.is_whitespace() => {
                    buffer.take_literal_or_keyword_or_identifier(&mut tokens);
                }
                ch => {
                    buffer.push(ch);

                    for (s, token) in &eager_tokens {
                        if buffer.take_from_end(s) {
                            buffer.take_literal_or_keyword_or_identifier(
                                &mut tokens,
                            );
                            tokens.push(token.clone());
                        }
                    }
                }
            },
            State::Comment => match ch {
                '\n' => {
                    tokens.push(Token::CommentLine {
                        line: buffer.take(),
                    });
                    state = State::Initial;
                }
                ch => {
                    buffer.push(ch);
                }
            },
        }
    }

    tokens
}

enum State {
    Initial,
    Comment,
}

#[derive(Default)]
struct Buffer {
    inner: String,
}

impl Buffer {
    pub fn push(&mut self, ch: char) {
        self.inner.push(ch);
    }

    pub fn take(&mut self) -> String {
        mem::take(&mut self.inner)
    }

    pub fn take_if_not_empty(&mut self) -> Option<String> {
        if self.inner.is_empty() {
            None
        } else {
            Some(self.take())
        }
    }

    pub fn take_literal_or_keyword_or_identifier(
        &mut self,
        tokens: &mut Vec<Token>,
    ) {
        tokens.extend(self.take_if_not_empty().map(|token| {
            if let Ok(value) = token.parse() {
                Token::IntegerLiteral { value }
            } else if token == "br" {
                Token::Keyword(Br)
            } else if token == "end" {
                Token::Keyword(End)
            } else if token == "fn" {
                Token::Keyword(Fn)
            } else {
                Token::Identifier { name: token }
            }
        }));
    }

    pub fn take_from_end(&mut self, s: &str) -> bool {
        if self.inner.ends_with(s) {
            self.inner.truncate(self.inner.len() - s.len());
            true
        } else {
            false
        }
    }
}
