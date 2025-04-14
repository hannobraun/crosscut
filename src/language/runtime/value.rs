use std::fmt;

use crate::language::code::{NodePath, display_tuple};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    Function {
        body: NodePath,
    },

    Integer {
        value: i32,
    },

    #[cfg(test)]
    Opaque {
        id: u32,
        display: &'static str,
    },

    Tuple {
        values: Vec<Value>,
    },
}

impl Value {
    pub fn nothing() -> Self {
        Self::Tuple { values: Vec::new() }
    }

    pub fn is_nothing(&self) -> bool {
        if let Self::Tuple { values } = self {
            values.is_empty()
        } else {
            false
        }
    }

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
            Self::Function { body } => {
                write!(f, "fn ")?;
                write!(f, "{}", body.hash())?;
            }

            Self::Integer { value } => {
                write!(f, "{value}")?;
            }

            #[cfg(test)]
            Self::Opaque { id: _, display } => {
                write!(f, "{display}")?;
            }

            Self::Tuple { values } => {
                display_tuple(values, f)?;
            }
        }

        Ok(())
    }
}
