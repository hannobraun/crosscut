use std::fmt;

use crate::language::packages::{FunctionId, Resolver};

use super::IntrinsicFunction;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    HostFunction { id: FunctionId },
    IntrinsicFunction { intrinsic: IntrinsicFunction },
}

impl Expression {
    pub fn display<'r>(
        &'r self,
        package: &'r Resolver,
    ) -> ExpressionDisplay<'r> {
        ExpressionDisplay {
            expression: self,
            package,
        }
    }
}

pub struct ExpressionDisplay<'r> {
    expression: &'r Expression,
    package: &'r Resolver,
}

impl fmt::Display for ExpressionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expression {
            Expression::HostFunction { id } => {
                let name = self.package.function_name_by_id(id);
                write!(f, "{name}")
            }
            Expression::IntrinsicFunction { intrinsic } => {
                write!(f, "{intrinsic}")
            }
        }
    }
}
