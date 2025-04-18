use crate::language::packages::Function;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub enum IntrinsicFunction {
    Add,
    Drop,
    Eval,
    Identity,
}

impl Function for IntrinsicFunction {
    fn name(&self) -> &str {
        match self {
            IntrinsicFunction::Add => "+",
            IntrinsicFunction::Drop => "drop",
            IntrinsicFunction::Eval => "eval",
            IntrinsicFunction::Identity => "identity",
        }
    }
}
