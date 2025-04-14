use std::fmt::{self};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Function,
    Integer,
    Tuple { values: Vec<Type> },
}

impl Type {
    pub fn nothing() -> Self {
        Self::Tuple { values: Vec::new() }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Function => {
                write!(f, "Function")?;
            }
            Self::Integer => {
                write!(f, "Integer")?;
            }
            Self::Tuple { values } => {
                display_tuple(values, f)?;
            }
        }

        Ok(())
    }
}

pub fn display_tuple<V>(values: &[V], f: &mut fmt::Formatter) -> fmt::Result
where
    V: fmt::Display,
{
    write!(f, "{{")?;

    for (i, value) in values.iter().enumerate() {
        if i == 0 || i + 1 == values.len() {
            write!(f, " ")?;
        } else {
            write!(f, ", ")?;
        }

        write!(f, "{value}")?;
    }

    write!(f, "}}")?;

    Ok(())
}
