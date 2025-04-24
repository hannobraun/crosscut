use crate::language::packages::Function;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub enum IntrinsicFunction {
    Add,
    Drop,
    Identity,
}

impl Function for IntrinsicFunction {
    fn name(&self) -> &str {
        match self {
            IntrinsicFunction::Add => "+",
            IntrinsicFunction::Drop => "drop",
            IntrinsicFunction::Identity => "identity",
        }
    }
}
