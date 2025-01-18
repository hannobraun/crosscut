use super::code::{
    Body, Code, Expression, FragmentId, FragmentKind, FunctionCallTarget,
    Literal,
};

#[derive(Debug)]
pub struct Interpreter {
    next: Option<FragmentId>,
    active_calls: Vec<ActiveCall>,
}

impl Interpreter {
    pub fn new(code: &Code) -> Self {
        let mut interpreter = Self {
            next: None,
            active_calls: Vec::new(),
        };
        interpreter.reset(code);

        interpreter
    }

    pub fn reset(&mut self, code: &Code) {
        let root = code.fragments().get(&code.root);
        self.next = root.body.entry().copied();
    }

    pub fn next(&self) -> Option<&FragmentId> {
        self.next.as_ref()
    }

    pub fn state(&self, code: &Code) -> InterpreterState {
        use InterpreterState::*;

        match self.next_expression(code) {
            NextExpression::Expression { .. } => Running,
            NextExpression::NoMoreFragments => Finished,
            NextExpression::NextFragmentIsNotAnExpression => Error,
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
                Expression::FunctionCall { target } => match target {
                    FunctionCallTarget::HostFunction { id } => {
                        if let Some(ActiveCall::ToHostFunction {
                            output: Some(output),
                            ..
                        }) = self.active_calls.last()
                        {
                            let output = output.clone();
                            self.active_calls.pop();
                            return self.evaluate_value(output);
                        } else {
                            self.active_calls.push(
                                ActiveCall::ToHostFunction {
                                    id: *id,
                                    fragment,
                                    output: None,
                                },
                            );
                            self.next = body.entry().copied();
                        }
                    }
                    FunctionCallTarget::IntrinsicFunction => {
                        todo!(
                            "Calls to intrinsic functions are not supported \
                            yet."
                        )
                    }
                },
                Expression::Literal {
                    literal: Literal::Integer { value },
                } => {
                    return self
                        .evaluate_value(Value::Integer { value: *value });
                }
            }
        }
    }

    fn evaluate_value(&mut self, value: Value) -> StepResult {
        let Some(ActiveCall::ToHostFunction {
            id,
            fragment,
            output,
        }) = self.active_calls.last_mut()
        else {
            self.next = None;
            return StepResult::Finished { output: value };
        };

        self.next = Some(*fragment);

        let output = output.insert(Value::Integer { value: 0 });
        StepResult::CallToHostFunction {
            id: *id,
            input: value,
            output,
        }
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

#[derive(Debug, Eq, PartialEq)]
pub enum InterpreterState {
    Running,
    Finished,
    Error,
}

#[derive(Debug, PartialEq)]
pub enum StepResult<'r> {
    CallToHostFunction {
        id: usize,
        input: Value,
        output: &'r mut Value,
    },
    Error,
    Finished {
        output: Value,
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

#[derive(Debug)]
enum ActiveCall {
    ToHostFunction {
        id: usize,
        fragment: FragmentId,
        output: Option<Value>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer { value: u32 },
}
