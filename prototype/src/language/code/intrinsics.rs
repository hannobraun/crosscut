use std::fmt;

use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntrinsicFunction {
    Identity,
    Literal { value: Value },
}

impl fmt::Display for IntrinsicFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Identity => {
                write!(f, "identity")?;
            }
            Self::Literal { value } => match value {
                Value::None => {}
                Value::Integer { value } => {
                    write!(f, "{value}")?;
                }
            },
        }

        Ok(())
    }
}
