use std::fmt;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub enum IntrinsicFunction {
    Drop,
    Eval,
    Identity,
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
        }

        Ok(())
    }
}
