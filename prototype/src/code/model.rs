use std::fmt;

#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    LiteralNumber { value: f64 },
    InvalidNumber { invalid: String },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::LiteralNumber { value } => write!(f, "{value}"),
            Expression::InvalidNumber { invalid } => {
                write!(f, "invalid number `{invalid}`")
            }
        }
    }
}
