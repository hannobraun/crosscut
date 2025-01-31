use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    UnresolvedIdentifier,
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodeError::UnresolvedIdentifier => {
                write!(f, "unresolved identifier")?;
            }
        }

        Ok(())
    }
}
