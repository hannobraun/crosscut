use super::tokenizer::{Token, Tokens};

pub struct Expressions(pub Vec<Expression>);

pub enum Expression {
    /// A word refers to a function or variable
    Word(String),
}

pub fn parse(mut tokens: Tokens) -> Result<Expressions, Error> {
    let mut expressions = Vec::new();

    while let Some(token) = tokens.next() {
        let expression = match token {
            Token::Word(word) => Expression::Word(word),
            token => return Err(Error::UnexpectedToken(token)),
        };

        expressions.push(expression);
    }

    Ok(Expressions(expressions))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected token: `{0:?}`")]
    UnexpectedToken(Token),
}
