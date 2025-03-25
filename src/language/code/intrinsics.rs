use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum IntrinsicFunction {
    Drop,
    Eval,
    Identity,
    Literal { literal: Literal },
}

impl IntrinsicFunction {
    pub fn resolve(name: &str) -> Option<Self> {
        match name {
            "drop" => Some(Self::Drop),
            "eval" => Some(Self::Eval),
            "identity" => Some(Self::Identity),
            _ => None,
        }
    }
}

impl fmt::Display for IntrinsicFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Drop => {
                write!(f, "drop")?;
            }
            Self::Eval => {
                write!(f, "eval")?;
            }
            Self::Identity => {
                write!(f, "identity")?;
            }
            Self::Literal { literal } => match literal {
                Literal::Function => {
                    write!(f, "fn")?;
                }
                Literal::Integer { value } => {
                    write!(f, "{value}")?;
                }
                Literal::Tuple => {
                    write!(f, "tuple")?;
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Literal {
    Function,
    Integer { value: i32 },
    Tuple,
}
