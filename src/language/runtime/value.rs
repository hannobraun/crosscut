use std::fmt;

use crate::language::code::{NodeHash, NodePath};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    Nothing,
    Function { body: NodeHash },
    Integer { value: i32 },
    Opaque { id: u32, display: &'static str },
    Tuple { elements: Vec<Value> },
}

impl Value {
    pub fn into_function_body(self) -> Result<NodePath, Self> {
        match self {
            Value::Function { body } => Ok(NodePath { hash: body }),
            _ => Err(self),
        }
    }
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
            Self::Tuple { elements } => {
                for element in elements {
                    write!(f, "{element} ")?;
                }
                write!(f, "tuple")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueWithSource {
    pub inner: Value,
}
