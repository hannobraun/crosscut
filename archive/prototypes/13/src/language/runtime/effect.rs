use crate::language::{code::Type, packages::FunctionId};

use super::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect {
    ProvidedFunction { id: FunctionId, input: Value },
    UnexpectedInput { expected: Type, actual: Value },
}
