use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Function,
    Integer,
    Nothing,
    Opaque { name: &'static str },
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Function => "Function",
            Self::Integer => "Integer",
            Self::Nothing => "Nothing",
            Self::Opaque { name } => name,
        };

        write!(f, "{text}")
    }
}
