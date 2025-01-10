use super::code::{Code, Expression, Fragment, Id};

pub struct Interpreter {
    pub next: Option<Id>,
    pub active_call: Option<usize>,
}

impl Interpreter {
    pub fn new(code: &Code) -> Self {
        Self {
            next: code.entry(),
            active_call: None,
        }
    }

    pub fn state(&self, code: &Code) -> &'static str {
        match self.next_expression(code) {
            NextExpression::Expression { .. } => "running",
            NextExpression::NoMoreFragments => "finished",
            NextExpression::NextFragmentIsNotAnExpression => "error",
        }
    }

    pub fn step(&mut self, code: &Code) -> InterpreterState {
        loop {
            let NextExpression::Expression { expression } =
                self.next_expression(code)
            else {
                // TASK: This isn't correct. It could also mean we're finished.
                return InterpreterState::Error;
            };

            match expression {
                Expression::FunctionCall { target, argument } => {
                    self.active_call = Some(*target);
                    self.next = Some(*argument);
                }
                Expression::LiteralValue { value } => {
                    if let Some(id) = self.active_call {
                        return InterpreterState::CallToHostFunction {
                            id,
                            input: *value,
                        };
                    } else {
                        self.next = None;
                        return InterpreterState::Finished { output: *value };
                    }
                }
            }
        }
    }

    pub fn reset(&mut self, code: &Code) {
        self.next = code.entry();
    }

    pub fn next_expression<'r>(&self, code: &'r Code) -> NextExpression<'r> {
        let Some(hash) = self.next else {
            return NextExpression::NoMoreFragments;
        };
        let fragment = code.fragment_by_id(&hash);
        let Fragment::Expression { expression } = fragment else {
            return NextExpression::NextFragmentIsNotAnExpression;
        };

        NextExpression::Expression { expression }
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpreterState {
    CallToHostFunction { id: usize, input: u32 },
    Error,
    Finished { output: u32 },
}

pub enum NextExpression<'r> {
    Expression { expression: &'r Expression },
    NoMoreFragments,
    NextFragmentIsNotAnExpression,
}
