use std::fmt;

use crate::language::packages::Function;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub enum IntrinsicFunction {
    Drop,
    Eval,
    Identity,
}

impl Function for IntrinsicFunction {
    fn name(&self) -> &str {
        match self {
            IntrinsicFunction::Drop => "drop",
            IntrinsicFunction::Eval => "eval",
            IntrinsicFunction::Identity => "identity",
        }
    }
}

impl fmt::Display for IntrinsicFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
