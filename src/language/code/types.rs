use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Integer,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Type::Integer => "Integer",
        };

        write!(f, "{text}")
    }
}
