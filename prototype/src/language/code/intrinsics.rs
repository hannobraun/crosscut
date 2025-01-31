use std::fmt;

use crate::language::runtime::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntrinsicFunction {
    Identity,
    Literal { value: Value },
}

impl IntrinsicFunction {
    pub fn resolve(name: &str) -> Option<Self> {
        match name {
            "identity" => Some(Self::Identity),
            _ => None,
        }
    }
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
