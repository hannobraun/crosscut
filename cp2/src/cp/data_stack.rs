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

    pub fn pop(&mut self) -> Result<Value, Error> {
        self.values.pop().ok_or(Error::PopFromEmptyStack)
    }

    pub fn pop_bool(&mut self) -> Result<bool, Error> {
        match self.pop()? {
            Value::Bool(bool) => Ok(bool),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Tried to pop value from empty stack")]
    PopFromEmptyStack,
}

#[derive(Debug)]
pub enum Value {
    Bool(bool),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
