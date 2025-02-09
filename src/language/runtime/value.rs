use std::fmt;

use crate::language::code::{NodeHash, NodePath};

#[derive(Clone, Copy, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    None,
    Function { hash: NodeHash },
    Integer { value: i32 },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Function { hash } => write!(f, "fn {}", hash),
            Self::Integer { value } => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValueWithSource {
    pub inner: Value,
    pub source: Option<NodePath>,
}
