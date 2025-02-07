use std::fmt;

use crate::language::packages::{FunctionId, Package};

use super::IntrinsicFunction;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    HostFunction { id: FunctionId },
    IntrinsicFunction { function: IntrinsicFunction },
}

impl Expression {
    pub fn display<'r>(
        &'r self,
        package: &'r Package,
    ) -> ExpressionDisplay<'r> {
        ExpressionDisplay {
            expression: self,
            package,
        }
    }
}

pub struct ExpressionDisplay<'r> {
    expression: &'r Expression,
    package: &'r Package,
}

impl fmt::Display for ExpressionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expression {
            Expression::HostFunction { id } => {
                let name = self.package.function_name_by_id(id);
                write!(f, "{name}")
            }
            Expression::IntrinsicFunction { function } => {
                write!(f, "{function}")
            }
        }
    }
}
