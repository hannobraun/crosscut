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
                        // We increment the code pointer unconditionally, even
                        // if we expect the program to be finished after this.
                        //
                        // This is important for two reasons:
                        //
                        // 1. If the program _is_ finished, then this fact can
                        //    be derived from the interpreter state, even if a
                        //    caller previously ignored the return value of this
                        //    function.
                        // 2. If the program is _not_ finished, then this is an
                        //    error, and we want the next call to the `step`
                        //    function to reflect that.
                        self.next_fragment += 1;

                        let Some(ActiveCall {
                            target: HostFunction { .. },
                        }) = self.active_call
                        else {
                            return InterpreterState::Finished {
                                output: *value,
                            };
                        };

                        self.active_call = None;

                        return InterpreterState::CallToHostFunction {
                            input: *value,
                        };
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
                    Token::LiteralNumber { .. } => {
                        return InterpreterState::Error;
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

#[derive(Debug, PartialEq)]
pub enum InterpreterState {
    CallToHostFunction {
        input: f64,
    },

    #[allow(unused)] // used only in test code, so far
    Error,

    #[allow(unused)] // used only in test code, so far
    Finished {
        output: f64,
    },

    Other,
}
