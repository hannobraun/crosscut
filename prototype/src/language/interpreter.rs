use super::code::{Code, Expression, HostFunction};

#[derive(Default)]
pub struct Interpreter {
    pub next_expression: usize,
    pub active_call: Option<ActiveCall>,
}

impl Interpreter {
    pub fn state(&self, code: &Code) -> &'static str {
        if self.next_expression(code).is_some() {
            "running"
        } else {
            "paused"
        }
    }

    pub fn step(&mut self, code: &Code) -> Option<(usize, f64)> {
        loop {
            let index = self.next_expression;
            let expression = self.next_expression(code)?;

            if let Some(ActiveCall {
                function: HostFunction { id },
            }) = self.active_call
            {
                match expression {
                    Expression::Identifier { .. } => {
                        // Function call is already in progress, and nested
                        // function calls are not supported yet.
                    }
                    Expression::LiteralNumber { value } => {
                        self.active_call = None;
                        self.next_expression += 1;
                        return Some((id, *value));
                    }
                }
            } else {
                match expression {
                    Expression::Identifier { .. } => {
                        if let Some(function) =
                            code.function_calls.get(&index).copied()
                        {
                            self.active_call = Some(ActiveCall { function });
                            self.next_expression += 1;
                            continue;
                        }

                        // No function found. This identifier is unresolved.
                    }
                    Expression::LiteralNumber { .. } => {
                        // There's not function call in progress, and thus
                        // nowhere to put a value right now.
                    }
                }
            }

            break;
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

pub struct ActiveCall {
    pub function: HostFunction,
}
