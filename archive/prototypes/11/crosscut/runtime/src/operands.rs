use alloc::vec::Vec;

use crate::Value;

#[derive(
    Clone, Debug, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize,
)]
pub struct Operands {
    values: Vec<Value>,
}

impl Operands {
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.values.push(value.into());
    }

    pub fn pop_any(&mut self) -> Result<Value, PopOperandError> {
        let value = self.values.pop().ok_or(PopOperandError::MissingOperand)?;
        Ok(value)
    }

    pub fn values(&self) -> impl Iterator<Item = Value> + '_ {
        self.values.iter().copied()
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    thiserror::Error,
)]
pub enum PopOperandError {
    #[error("Missing operand")]
    MissingOperand,
}
