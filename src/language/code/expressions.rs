use std::fmt;

use crate::language::packages::{FunctionId, Packages};

use super::{IntrinsicFunction, Literal};

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    HostFunction { id: FunctionId },
    IntrinsicFunction { intrinsic: IntrinsicFunction },
    Literal { literal: Literal },
}

impl Expression {
    pub fn display<'r>(
        &'r self,
        packages: &'r Packages,
    ) -> ExpressionDisplay<'r> {
        ExpressionDisplay {
            expression: self,
            packages,
        }
    }
}

pub struct ExpressionDisplay<'r> {
    expression: &'r Expression,
    packages: &'r Packages,
}

impl fmt::Display for ExpressionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expression {
            Expression::HostFunction { id } => {
                let name = self.packages.function_name_by_id(id);
                write!(f, "{name}")
            }
            Expression::IntrinsicFunction { intrinsic } => {
                write!(f, "{intrinsic}")
            }
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
