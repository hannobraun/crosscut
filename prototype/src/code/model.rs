use std::fmt;

#[derive(Debug)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for expression in &self.expressions {
            writeln!(f, "{expression}")?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Expression {
    LiteralNumber { value: f64 },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::LiteralNumber { value } => write!(f, "{value}"),
        }
    }
}
