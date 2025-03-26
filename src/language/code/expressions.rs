use std::fmt;

use crate::language::packages::Packages;

use super::Literal;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    Literal { literal: Literal },
}

impl Expression {
    pub fn display<'r>(&'r self, _: &'r Packages) -> ExpressionDisplay<'r> {
        ExpressionDisplay { expression: self }
    }
}

pub struct ExpressionDisplay<'r> {
    expression: &'r Expression,
}

impl fmt::Display for ExpressionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expression {
            Expression::Literal { literal } => match literal {
                Literal::Function => {
                    write!(f, "fn")
                }
                Literal::Integer { value } => {
                    write!(f, "{value}")
                }
                Literal::Tuple => {
                    write!(f, "tuple")
                }
            },
        }
    }
}
