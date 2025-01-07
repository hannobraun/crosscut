use std::collections::BTreeMap;

use super::code::{Code, Expression, FunctionType};

pub struct Interpreter {
    pub functions: BTreeMap<String, FunctionType>,
    pub next_expression: usize,
    pub active_function: Option<FunctionType>,
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
        let index = self.next_expression;
        let expression = self.next_expression(code)?;

        if let Some(FunctionType {
            input: (),
            output: (),
        }) = self.active_function
        {
            match expression {
                Expression::Identifier { .. } => {
                    // Function call is already in progress, and nested function
                    // calls are not supported yet.
                }
                Expression::LiteralNumber { value } => {
                    self.active_function = None;
                    self.next_expression += 1;
                    return Some(*value);
                }
            }
        } else {
            match expression {
                Expression::Identifier { .. } => {
                    if let Some(function) =
                        code.function_calls.get(&index).copied()
                    {
                        self.active_function = Some(function);
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
