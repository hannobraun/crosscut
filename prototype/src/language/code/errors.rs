use std::fmt;

use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    UnresolvedIdentifier { candidates: Vec<Expression> },
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodeError::UnresolvedIdentifier { candidates: _ } => {
                write!(f, "unresolved identifier")?;
            }
        }

        Ok(())
    }
}
