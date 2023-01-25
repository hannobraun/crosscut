use std::iter;

use super::tokenizer::Token;

pub enum SyntaxTree {
    /// A function
    Fn { name: String },
}

pub fn parse(
    tokens: impl Iterator<Item = Token>,
) -> impl Iterator<Item = SyntaxTree> {
    parse_tokens(tokens)
}

fn parse_tokens(
    mut tokens: impl Iterator<Item = Token>,
) -> impl Iterator<Item = SyntaxTree> {
    iter::from_fn(move || {
        for token in &mut tokens {
            match token {
                Token::Fn { name } => {
                    return Some(SyntaxTree::Fn { name });
                }
                Token::ArrayOpen => {}
                Token::ArrayClose => {}
            }
        }

        None
    })
}
