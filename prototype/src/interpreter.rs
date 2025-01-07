use std::collections::BTreeSet;

use crate::code::{Code, Expression};

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

    pub fn step(&mut self, code: &Code) -> Option<f64> {
        let expression = code.expressions.get(self.next_expression)?;

        match expression {
            Expression::Identifier { name } => {
                if self.functions.contains(name) && !self.active_function {
                    self.active_function = true;
                    self.next_expression += 1;
                }
            }
            Expression::LiteralNumber { value } => {
                if self.active_function {
                    self.next_expression += 1;

                    return Some(*value);
                }
            }
        }

        None
    }
}
