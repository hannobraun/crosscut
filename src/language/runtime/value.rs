use std::fmt;

use crate::language::code::{NodeHash, NodePath};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    Nothing,
    Function { body: NodeHash },
    Integer { value: i32 },
    Opaque { id: u32, display: &'static str },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nothing => {
                write!(f, "nothing")?;
            }
            Self::Function { body } => {
                write!(f, "fn ")?;
                write!(f, "{}", body)?;
            }
            Self::Integer { value } => {
                write!(f, "{value}")?;
            }
            Self::Opaque { id: _, display } => {
                write!(f, "{display}")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueWithSource {
    pub inner: Value,
    pub source: Option<NodePath>,
}

impl ValueWithSource {
    pub fn into_function_body(self) -> Result<NodePath, Self> {
        match self.inner {
            Value::Function { body } => Ok(NodePath { hash: body }),
            _ => Err(self),
        }
    }
}
