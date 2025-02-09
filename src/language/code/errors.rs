use std::fmt;

use super::Expression;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    UnresolvedIdentifier { candidates: Vec<Expression> },
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnresolvedIdentifier { candidates } => {
                write!(f, "unresolved syntax node")?;

                if !candidates.is_empty() {
                    write!(f, " (could resolve to")?;

                    for (i, candidate) in candidates.iter().enumerate() {
                        if i == 0 {
                            write!(f, " ")?;
                        } else {
                            write!(f, ", ")?;
                        }

                        match candidate {
                            Expression::HostFunction { .. } => {
                                write!(f, "host function")?;
                            }
                            Expression::IntrinsicFunction { .. } => {
                                write!(f, "intrinsic function")?;
                            }
                        }
                    }

                    write!(f, ")")?;
                }
            }
        }

        Ok(())
    }
}
