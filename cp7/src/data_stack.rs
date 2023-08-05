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
        self.values.pop().ok_or(DataStackError {
            kind: DataStackErrorKind::StackIsEmpty,
        })
    }

    pub fn pop_number(&mut self) -> DataStackResult<Number> {
        let value = self.pop_any()?;
        let Value::Number(number) = value;
        Ok(number)
    }
}

pub enum Value {
    Number(Number),
}

impl From<Number> for Value {
    fn from(number: Number) -> Self {
        Self::Number(number)
    }
}

pub type Number = i64;

pub type DataStackResult<T> = Result<T, DataStackError>;

#[derive(Debug, thiserror::Error)]
#[error("Stack is empty")]
pub struct DataStackError {
    pub kind: DataStackErrorKind,
}

#[derive(Debug)]
pub enum DataStackErrorKind {
    StackIsEmpty,
}
