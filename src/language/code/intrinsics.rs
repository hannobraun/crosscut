use std::fmt;

use crate::language::runtime::Value;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum IntrinsicFunction {
    Identity,
    Literal { literal: Literal },
}

impl IntrinsicFunction {
    pub fn resolve(name: &str) -> Option<Self> {
        if let Ok(value) = name.parse() {
            Some(IntrinsicFunction::Literal {
                literal: Literal {
                    value: Value::Integer { value },
                },
            })
        } else {
            match name {
                "identity" => Some(Self::Identity),
                _ => None,
            }
        }
    }
}

impl fmt::Display for IntrinsicFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Identity => {
                write!(f, "identity")?;
            }
            Self::Literal { literal: value } => match value.value {
                Value::Nothing => {}
                Value::Function { hash: _ } => {
                    writeln!(f, "fn")?;
                }
                Value::Integer { value } => {
                    write!(f, "{value}")?;
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Literal {
    pub value: Value,
}
