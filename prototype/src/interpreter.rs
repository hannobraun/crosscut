use std::collections::BTreeSet;

use crate::code::Code;

pub struct Interpreter {
    pub functions: BTreeSet<String>,
    pub next_expression: usize,
    pub active_function: bool,
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
