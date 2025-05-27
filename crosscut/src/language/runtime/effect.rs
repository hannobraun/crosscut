use crate::language::code::Type;

use super::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    ApplyProvidedFunction { name: String, input: Value },
    ProvidedFunctionNotFound,
    UnexpectedInput { expected: Type, actual: Value },
}
