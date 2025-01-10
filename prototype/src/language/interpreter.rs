use super::code::{Body, Code, Expression, FragmentId, FragmentKind};

pub struct Interpreter {
    next: Option<FragmentId>,
    active_call: Option<ActiveCall>,
}

impl Interpreter {
    pub fn new(code: &Code) -> Self {
        Self {
            next: code.root.entry().copied(),
            active_call: None,
        }
    }

    pub fn next(&self) -> Option<&FragmentId> {
        self.next.as_ref()
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
            let NextExpression::Expression { expression, body } =
                self.next_expression(code)
            else {
                return InterpreterState::Error;
            };

            match expression {
                Expression::FunctionCall { target } => {
                    self.active_call =
                        Some(ActiveCall::ToHostFunction { id: *target });
                    self.next = body.entry().copied();
                }
                Expression::LiteralValue { value } => {
                    if let Some(ActiveCall::ToHostFunction { id }) =
                        self.active_call
                    {
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
        self.next = code.root.entry().copied();
    }

    pub fn next_expression<'r>(&self, code: &'r Code) -> NextExpression<'r> {
        let Some(id) = self.next else {
            return NextExpression::NoMoreFragments;
        };
        let fragment = code.fragments().get(&id);
        let FragmentKind::Expression { expression } = &fragment.kind else {
            return NextExpression::NextFragmentIsNotAnExpression;
        };

        NextExpression::Expression {
            expression,
            body: &fragment.body,
        }
    }
}

pub enum ActiveCall {
    ToHostFunction { id: usize },
}

#[derive(Debug, PartialEq)]
pub enum InterpreterState {
    CallToHostFunction { id: usize, input: u32 },
    Error,
    Finished { output: u32 },
}

pub enum NextExpression<'r> {
    Expression {
        expression: &'r Expression,
        body: &'r Body,
    },
    NoMoreFragments,
    NextFragmentIsNotAnExpression,
}
