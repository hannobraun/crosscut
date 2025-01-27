#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Codebase {
    pub value: Option<i32>,
}

impl Codebase {
    pub fn new() -> Self {
        Self { value: None }
    }
}
