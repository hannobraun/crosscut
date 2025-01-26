use std::collections::VecDeque;

use super::{tokenize::tokenize, Token};

/// # The tokens in a script
///
/// See [parent module](super).
pub struct Tokens {
    inner: VecDeque<Token>,
}

impl Tokens {
    /// # Tokenize the provided input
    ///
    /// Takes raw text, as input by the developer, and creates its tokenized
    /// form.
    pub fn tokenize(input: &str) -> Self {
        let tokens = tokenize(input);
        Self {
            inner: tokens.into(),
        }
    }

    /// # Peek at the next token without taking it
    pub fn peek(&self) -> Result<&Token, NoMoreTokens> {
        self.inner.front().ok_or(NoMoreTokens)
    }

    /// # Take the next token
    pub fn take(&mut self) -> Result<Token, NoMoreTokens> {
        self.inner.pop_front().ok_or(NoMoreTokens)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("No more tokens")]
pub struct NoMoreTokens;
