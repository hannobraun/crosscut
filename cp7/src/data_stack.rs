use std::fmt;

use enum_variant_type::EnumVariantType;

use crate::pipeline::c_parser::SyntaxTree;

pub struct DataStack {
    values: Vec<Value>,
}

impl DataStack {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.values.push(value.into())
    }

    pub fn pop_any(&mut self) -> DataStackResult<Value> {
        self.pop_inner("any value")
    }

    pub fn pop_number(&mut self) -> DataStackResult<value::Number> {
        let ty = "number";

        let value = self.pop_inner(ty)?;
        let number = value.expect(ty)?;

        Ok(number)
    }

    fn pop_inner(&mut self, expected: &'static str) -> DataStackResult<Value> {
        self.values
            .pop()
            .ok_or(DataStackError::StackIsEmpty { expected })
    }
}

#[derive(Debug, EnumVariantType)]
#[evt(module = "value", derive(Debug))]
pub enum Value {
    Block(SyntaxTree),
    Number(i64),
    Symbol(String),
}

impl Value {
    pub fn expect<T>(self, expected: &'static str) -> Result<T, TypeError>
    where
        T: TryFrom<Value, Error = Value>,
    {
        self.try_into()
            .map_err(|value| TypeError { value, expected })
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Block(block) => write!(f, "{{ {block:?} }}"),
            Value::Number(number) => write!(f, "{number}"),
            Value::Symbol(symbol) => write!(f, ":{symbol}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Expected {expected}, found `{value}`")]
pub struct TypeError {
    pub value: Value,
    pub expected: &'static str,
}

pub type DataStackResult<T> = Result<T, DataStackError>;

#[derive(Debug, thiserror::Error)]
pub enum DataStackError {
    #[error("Stack is empty (expected {expected})")]
    StackIsEmpty { expected: &'static str },

    #[error("Unexpected value")]
    UnexpectedValue(#[from] TypeError),
}
