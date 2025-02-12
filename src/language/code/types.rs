use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Integer,
    Nothing,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Integer => "Integer",
            Self::Nothing => "Nothing",
        };

        write!(f, "{text}")
    }
}
