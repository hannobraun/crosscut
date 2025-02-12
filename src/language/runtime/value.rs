use std::fmt;

use crate::language::code::{NodeHash, NodePath};

#[derive(Clone, Copy, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    Nothing,
    Function { hash: NodeHash },
    Integer { value: i32 },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Nothing => write!(f, "nothing")?,
            Self::Function { hash } => write!(f, "fn {}", hash)?,
            Self::Integer { value } => write!(f, "{value}")?,
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValueWithSource {
    pub inner: Value,
    pub source: Option<NodePath>,
}

impl ValueWithSource {
    pub fn into_function_body(self) -> Result<NodePath, Self> {
        match self.inner {
            Value::Function { hash } => Ok(NodePath { hash }),
            _ => Err(self),
        }
    }
}
