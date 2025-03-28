use std::fmt;

use crate::language::code::NodePath;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    Nothing,
    Function { body: NodePath },
    Integer { value: i32 },
    Opaque { id: u32, display: &'static str },
    Tuple { elements: Vec<Value> },
}

impl Value {
    pub fn into_function_body(self) -> Result<NodePath, Self> {
        match self {
            Value::Function { body } => Ok(body),
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
                write!(f, "{}", body.hash())?;
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
