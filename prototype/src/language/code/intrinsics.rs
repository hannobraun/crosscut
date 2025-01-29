use crate::language::interpreter::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IntrinsicFunction {
    Identity,
    Literal { value: Value },
}
