use std::fmt;

use crate::language::host::Host;

use super::IntrinsicFunction;

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    HostFunction { id: u32 },
    IntrinsicFunction { function: IntrinsicFunction },
}

impl Expression {
    pub fn display<'r>(&'r self, host: &'r Host) -> ExpressionDisplay<'r> {
        ExpressionDisplay {
            expression: self,
            host,
        }
    }
}

pub struct ExpressionDisplay<'r> {
    expression: &'r Expression,
    host: &'r Host,
}

impl fmt::Display for ExpressionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expression {
            Expression::HostFunction { id } => {
                let name = self.host.function_name_by_id(id);
                write!(f, "{name}")
            }
            Expression::IntrinsicFunction { function } => {
                write!(f, "{function}")
            }
        }
    }
}
