use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Function,
    Integer,
    Nothing,
    Opaque { name: &'static str },
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function => write!(f, "Function")?,
            Self::Integer => write!(f, "Integer")?,
            Self::Nothing => write!(f, "Nothing")?,
            Self::Opaque { name } => write!(f, "{name}")?,
        }

        Ok(())
    }
}
