use std::mem;

use map_macro::map;

use crate::cp::{
    keywords::Keyword,
    tokens::{Token, STRING_DELIMITER},
};

#[derive(Debug)]
pub enum Tokenizer {
    Searching,
    ProcessingAny { buf: String },
    ProcessingString { buf: String },
}

pub fn new() -> Tokenizer {
    Tokenizer::Searching
}

pub fn tokenize(code: impl IntoIterator<Item = char>) -> Vec<Token> {
    let code = code.into_iter();
    let mut tokens = Vec::new();

    let mut tokenizer = new();

    for ch in code {
        tokenizer = push_char(ch, tokenizer, &mut tokens);
    }
    tokens.extend(finalize(tokenizer));

    tokens
}

pub fn push_char(
    ch: char,
    tokenizer: Tokenizer,
    tokens: &mut Vec<Token>,
) -> Tokenizer {
    match tokenizer {
        Tokenizer::Searching => {
            if ch.is_whitespace() {
                return tokenizer;
            }

            if ch == STRING_DELIMITER {
                return Tokenizer::ProcessingString { buf: String::new() };
            }

            let buf = String::from(ch);
            Tokenizer::ProcessingAny { buf }
        }
        Tokenizer::ProcessingAny { mut buf } => {
            let t = match_eagerly(&buf);
            if !t.is_empty() {
                tokens.extend(t);
                buf.clear();

                if ch.is_whitespace() {
                    return Tokenizer::Searching;
                }
            }

            if ch == STRING_DELIMITER {
                tokens.extend(match_delimited(&buf));
                return Tokenizer::ProcessingString { buf: String::new() };
            }

            if !ch.is_whitespace() {
                buf.push(ch);
                return Tokenizer::ProcessingAny { buf };
            }

            if !buf.is_empty() {
                tokens.extend(match_delimited(&buf));
                return Tokenizer::Searching;
            }

            Tokenizer::ProcessingAny { buf }
        }
        Tokenizer::ProcessingString { mut buf } => {
            if ch == STRING_DELIMITER {
                let string = mem::take(&mut buf);
                tokens.push(Token::String(string));
                return Tokenizer::Searching;
            }

            buf.push(ch);

            Tokenizer::ProcessingString { buf }
        }
    }
}

pub fn finalize(tokenizer: Tokenizer) -> Vec<Token> {
    if let Tokenizer::ProcessingAny { buf } = tokenizer {
        return match_delimited(&buf);
    }

    vec![]
}

fn match_eagerly(buf: &str) -> Vec<Token> {
    let mut delimiters = map! {
        "=>" => Token::BindingOperator,
        "." => Token::Period,
        "{" => Token::CurlyBracketOpen,
        "}" => Token::CurlyBracketClose,
        "(" => Token::RoundBracketOpen,
        ")" => Token::RoundBracketClose,
        "[" => Token::SquareBracketOpen,
        "]" => Token::SquareBracketClose,
    };

    if let Some(token) = delimiters.remove(buf) {
        return vec![token];
    }

    for (delimiter, token) in delimiters {
        if let Some((first_token, "")) = buf.split_once(delimiter) {
            let mut tokens = match_delimited(first_token);
            tokens.push(token);
            return tokens;
        }
    }

    vec![]
}

fn match_delimited(buf: &str) -> Vec<Token> {
    let mut tokens = match_eagerly(buf);
    if !tokens.is_empty() {
        return tokens;
    }

    if let Some(keyword) = Keyword::parse(buf) {
        tokens.push(Token::Keyword(keyword));
        return tokens;
    }
    if let Some(("", symbol)) = buf.split_once(':') {
        tokens.push(Token::Symbol(symbol.into()));
        return tokens;
    }

    tokens.push(Token::Ident(buf.into()));
    tokens
}
