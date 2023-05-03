use super::a_tokenizer::{Token, Tokenizer};

pub struct Parser {
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self { tokenizer }
    }

    pub async fn next_token(
        &mut self,
    ) -> Result<Option<SyntaxElement>, ParserError> {
        let Some(token) = self.tokenizer.next_token().await else {
            return Ok(None);
        };

        match token {
            Token::Ident(ident) => Ok(Some(SyntaxElement::Word(ident))),
            token => Err(ParserError::UnexpectedToken(token)),
        }
    }
}

pub enum SyntaxElement {
    Word(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
}
