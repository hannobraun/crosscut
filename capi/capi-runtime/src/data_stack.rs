#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct DataStack {
    values: Vec<Value>,
    saved: Vec<usize>,
}

impl DataStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clone(&mut self) -> Value {
        self.values.last().copied().unwrap()
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Result<usize, PopFromEmptyStack> {
        self.values
            .pop()
            .ok_or(PopFromEmptyStack)
            .map(|value| value.0)
    }

    pub fn num_values(&self) -> usize {
        self.values.len()
    }

    pub fn save(&mut self, num: usize) {
        for _ in 0..num {
            let value = self.pop().unwrap();
            self.saved.push(value);
        }
    }

    pub fn restore(&mut self) {
        while let Some(x) = self.saved.pop() {
            self.push(Value(x));
        }
    }
}

#[derive(Copy, Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Value(pub usize);

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    thiserror::Error,
)]
#[error("Tried to pop value from empty stack")]
pub struct PopFromEmptyStack;
