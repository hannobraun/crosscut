#[derive(Debug, Eq, PartialEq)]
pub enum StepResult {
    Finished { output: Value },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    None,
    Integer { value: i32 },
}
