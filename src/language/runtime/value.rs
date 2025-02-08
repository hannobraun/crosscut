use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Value {
    None,
    Integer { value: i32 },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Integer { value } => write!(f, "{value}"),
        }
    }
}
