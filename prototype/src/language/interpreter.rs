use super::code::{Code, Expression, Fragment, HostFunction, Token};

#[derive(Default)]
pub struct Interpreter {
    pub next_fragment: usize,
    pub active_call: Option<ActiveCall>,
}

impl Interpreter {
    pub fn state(&self, code: &Code) -> &'static str {
        if self.next_fragment(code).is_some() {
            "running"
        } else {
            "paused"
        }
    }

    pub fn step(&mut self, code: &Code) -> InterpreterState {
        loop {
            let index = self.next_fragment;
            let Some(fragment) = self.next_fragment(code) else {
                return InterpreterState::Other;
            };

            match fragment {
                Fragment::Expression { expression } => match expression {
                    Expression::LiteralValue { value } => {
                        if let Some(ActiveCall {
                            target: HostFunction { id },
                        }) = self.active_call
                        {
                            self.active_call = None;
                            self.next_fragment += 1;

                            return InterpreterState::CallToHostFunction {
                                id,
                                input: *value,
                            };
                        } else {
                            // There's no function call in progress, and thus
                            // nowhere to put a value right now.
                        }
                    }
                },
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
                            self.next_fragment += 1;
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
                            self.next_fragment += 1;

                            return InterpreterState::CallToHostFunction {
                                id,
                                input: *value,
                            };
                        } else {
                            // There's no function call in progress, and thus
                            // nowhere to put a value right now.
                        }
                    }
                },
            }

            break;
        }

        InterpreterState::Other
    }

    pub fn next_fragment<'r>(&self, code: &'r Code) -> Option<&'r Fragment> {
        let fragment = code.fragments.get(self.next_fragment)?;
        Some(fragment)
    }
}

pub struct ActiveCall {
    pub target: HostFunction,
}

pub enum InterpreterState {
    CallToHostFunction {
        #[allow(unused)] // used only in test code, so far
        id: usize,
        input: f64,
    },
    Other,
}
