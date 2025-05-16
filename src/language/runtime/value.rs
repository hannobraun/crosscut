use std::fmt;

use crate::language::code::{NodePath, display_tuple};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    Function { parameter: String, body: NodePath },
    Integer { value: i32 },
    ProvidedFunction { name: String },
    Tuple { values: Vec<Value> },
}

impl Value {
    pub fn nothing() -> Self {
        Self::Tuple { values: Vec::new() }
    }

    #[cfg(test)]
    pub fn is_nothing(&self) -> bool {
        if let Self::Tuple { values } = self {
            values.is_empty()
        } else {
            false
        }
    }

    pub fn into_function_body(self) -> Result<NodePath, Self> {
        match self {
            Value::Function { parameter: _, body } => Ok(body),
            _ => Err(self),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function { parameter, body } => {
                write!(f, "fn {parameter}: {}", body.hash())?;
            }
            Self::Integer { value } => {
                write!(f, "{value}")?;
            }
            Self::ProvidedFunction { name } => {
                write!(f, "provided function `{name}`")?;
            }
            Self::Tuple { values } => {
                display_tuple(values, f)?;
            }
        }

        Ok(())
    }
}
