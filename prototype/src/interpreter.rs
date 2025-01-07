use std::collections::BTreeSet;

use crate::code::{Code, Expression};

pub struct Interpreter {
    pub functions: BTreeSet<String>,
    pub next_expression: usize,
    pub active_function: bool,
}

impl Interpreter {
    pub fn state(&self, code: &Code) -> &'static str {
        if self.next_expression(code).is_some() {
            "running"
        } else {
            "paused"
        }
    }

    pub fn step(&mut self, code: &Code) -> Option<f64> {
        let expression = self.next_expression(code)?;

        if self.active_function {
            match expression {
                Expression::Identifier { .. } => {
                    // Function call is already in progress, and nested function
                    // calls are not supported yet.
                }
                Expression::LiteralNumber { value } => {
                    self.next_expression += 1;
                    return Some(*value);
                }
            }
        } else {
            match expression {
                Expression::Identifier { name } => {
                    if self.functions.contains(name) {
                        self.active_function = true;
                        self.next_expression += 1;
                    }
                }
                Expression::LiteralNumber { .. } => {
                    // There's not function call in progress, and thus nowhere
                    // to put a value right now.
                }
            }
        }

        None
    }

    pub fn next_expression<'r>(
        &self,
        code: &'r Code,
    ) -> Option<&'r Expression> {
        code.expressions.get(self.next_expression)
    }
}
