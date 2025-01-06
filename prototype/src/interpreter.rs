use crate::code::Code;

#[derive(Default)]
pub struct Interpreter {
    pub next_expression: usize,
}

impl Interpreter {
    pub fn state(&self, code: &Code) -> &'static str {
        if self.next_expression >= code.expressions.len() {
            "paused"
        } else {
            "running"
        }
    }
}
