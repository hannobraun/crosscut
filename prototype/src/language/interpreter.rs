use super::code::{Code, Fragment, HostFunction, Token};

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

            match expression {
                Fragment::UnexpectedToken { token } => match token {
                    Token::Identifier { .. } => {
                        if let Some(ActiveCall {
                            target: HostFunction { id: _ },
                        }) = self.active_call
                        {
                            // Function call is already in progress, and nested
                            // function calls are not supported yet.
                        } else if let Some(target) =
                            code.function_calls.get(&index).copied()
                        {
                            self.active_call = Some(ActiveCall { target });
                            self.next_expression += 1;
                            continue;
                        } else {
                            // No function found. This identifier is unresolved.
                        }
                    }
                    Token::LiteralNumber { value } => {
                        if let Some(ActiveCall {
                            target: HostFunction { id },
                        }) = self.active_call
                        {
                            self.active_call = None;
                            self.next_expression += 1;
                            return Some((id, *value));
                        } else {
                            // There's not function call in progress, and thus
                            // nowhere to put a value right now.
                        }
                    }
                },
            }

            break;
        }

        None
    }

    pub fn next_expression<'r>(&self, code: &'r Code) -> Option<&'r Fragment> {
        let fragment = code.fragments.get(self.next_expression)?;
        Some(fragment)
    }
}

pub struct ActiveCall {
    pub target: HostFunction,
}
