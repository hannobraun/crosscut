use super::tokenizer::{ExpectedToken, NoMoreTokens, Token, Tokens};

#[derive(Clone, Debug)]
pub struct Expressions(pub Vec<Expression>);

#[derive(Clone, Debug)]
pub enum Expression {
    /// Binds values from the stack to provided names
    Binding(Vec<String>),

    Array(Expressions),

    /// A block of code that is lazily evaluated
    Block(Expressions),

    /// A word refers to a function or variable
    Word(String),
}

pub fn parse(mut tokens: Tokens) -> Result<Expressions, Error> {
    let mut expressions = Vec::new();

    while let Ok(token) = tokens.next() {
        let expression = parse_expression(token, &mut tokens)?;
        expressions.push(expression);
    }

    Ok(Expressions(expressions))
}

fn parse_expression(
    token: Token,
    tokens: &mut Tokens,
) -> Result<Expression, Error> {
    match token {
        Token::BindingOperator => {
            let binding_names = parse_binding(tokens)?;
            Ok(Expression::Binding(binding_names))
        }
        Token::CurlyBracketOpen => {
            let expressions = parse_block(tokens)?;
            Ok(Expression::Block(expressions))
        }
        Token::SquareBracketOpen => parse_array(tokens),
        Token::Ident(ident) => Ok(Expression::Word(ident)),
        token => Err(Error::UnexpectedToken(token)),
    }
}

fn parse_binding(tokens: &mut Tokens) -> Result<Vec<String>, Error> {
    let mut binding_names = Vec::new();

    loop {
        match tokens.next()? {
            Token::Ident(ident) => binding_names.push(ident),
            Token::Period => break,
            token => return Err(Error::UnexpectedToken(token)),
        }
    }

    Ok(binding_names)
}

fn parse_block(tokens: &mut Tokens) -> Result<Expressions, Error> {
    let mut expressions = Vec::new();

    loop {
        let expression = match tokens.next()? {
            Token::CurlyBracketClose => break,
            token => parse_expression(token, tokens)?,
        };

        expressions.push(expression);
    }

    Ok(Expressions(expressions))
}

fn parse_array(tokens: &mut Tokens) -> Result<Expression, Error> {
    let mut expressions = Vec::new();

    loop {
        let expression = match tokens.next()? {
            Token::SquareBracketClose => break,
            token => parse_expression(token, tokens)?,
        };

        expressions.push(expression)
    }

    Ok(Expression::Array(Expressions(expressions)))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Expected more tokens")]
    ExpectedMoreTokens(#[from] NoMoreTokens),

    #[error(transparent)]
    ExpectedToken(#[from] ExpectedToken),

    #[error("Unexpected token: `{0:?}`")]
    UnexpectedToken(Token),
}
