use std::fmt;

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

pub fn display_tuple(values: &[Type], f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{")?;

    for (i, value) in values.iter().enumerate() {
        if i == 0 || i == values.len() - 1 {
            write!(f, " ")?;
        } else {
            write!(f, ", ")?;
        }

        write!(f, "{value}")?;
    }

    write!(f, "}}")?;

    Ok(())
}
