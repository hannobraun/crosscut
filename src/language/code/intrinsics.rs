use std::fmt;

use crate::language::runtime::Value;

use super::{Codebase, NodeHash, NodePath};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum IntrinsicFunction {
    Identity,
    Literal { literal: Literal },
}

impl IntrinsicFunction {
    pub fn resolve(name: &str) -> Option<Self> {
        if let Ok(value) = name.parse() {
            Some(IntrinsicFunction::Literal {
                literal: Literal::Integer { value },
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
            Self::Literal { literal } => match literal {
                Literal::Function { hash: _ } => {
                    writeln!(f, "fn")?;
                }
                Literal::Integer { value } => {
                    write!(f, "{value}")?;
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Literal {
    Function { hash: NodeHash },
    Integer { value: i32 },
}

impl Literal {
    pub fn to_value(&self, path: &NodePath, codebase: &Codebase) -> Value {
        match *self {
            Self::Function { hash: _ } => {
                let Some(child) = codebase.child_of(path) else {
                    unreachable!(
                        "Function literal must have a child, or it wouldn't \
                        have been resolved as a function literal."
                    );
                };

                Value::Function {
                    hash: *child.hash(),
                }
            }
            Self::Integer { value } => Value::Integer { value },
        }
    }
}
