use crate::language::code::Type;

use super::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    ProvidedFunction { name: String, input: Value },
    UnexpectedInput { expected: Type, actual: Value },
}
