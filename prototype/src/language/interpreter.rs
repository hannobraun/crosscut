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

    pub fn step(&mut self, code: &Code) -> StepResult {
        loop {
            let NextExpression::Expression {
                expression,
                body,
                fragment,
            } = self.next_expression(code)
            else {
                return StepResult::Error;
            };

            match expression {
                Expression::FunctionCall { target } => {
                    if let Some(ActiveCall::ToHostFunction {
                        output: Some(output),
                        ..
                    }) = self.active_call
                    {
                        self.active_call = None;
                        return self.evaluate_value(output);
                    } else {
                        self.active_call = Some(ActiveCall::ToHostFunction {
                            id: *target,
                            fragment,
                            output: None,
                        });
                        self.next = body.entry().copied();
                    }
                }
                Expression::LiteralValue { value } => {
                    return self.evaluate_value(*value);
                }
            }
        }
    }

    fn evaluate_value(&mut self, value: u32) -> StepResult {
        if let Some(ActiveCall::ToHostFunction {
            id,
            fragment,
            output,
        }) = &mut self.active_call
        {
            self.next = Some(*fragment);
            StepResult::CallToHostFunction {
                id: *id,
                input: value,
                output: output.insert(0),
            }
        } else {
            self.next = None;
            StepResult::Finished { output: value }
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
            fragment: id,
        }
    }
}

pub enum ActiveCall {
    ToHostFunction {
        id: usize,
        fragment: FragmentId,
        output: Option<u32>,
    },
}

#[derive(Debug, PartialEq)]
pub enum StepResult<'r> {
    CallToHostFunction {
        id: usize,
        input: u32,
        output: &'r mut u32,
    },
    Error,
    Finished {
        output: u32,
    },
}

pub enum NextExpression<'r> {
    Expression {
        expression: &'r Expression,
        body: &'r Body,
        fragment: FragmentId,
    },
    NoMoreFragments,
    NextFragmentIsNotAnExpression,
}
