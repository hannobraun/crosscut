use super::code::{
    Body, Codebase, Expression, FragmentKind, FunctionCallTarget, Literal,
    NodeId,
};

#[derive(Debug)]
pub struct Interpreter {
    next: Option<NodeId>,
    active_calls: Vec<ActiveCall>,
}

impl Interpreter {
    pub fn new(code: &Codebase) -> Self {
        let next = code.root().node.body.entry().copied();

        Self {
            next,
            active_calls: Vec::new(),
        }
    }

    pub fn reset(&mut self, code: &Codebase) {
        // Let's use re-use the constructor instead of trying to do anything
        // more advanced (and possibly efficient) here. That way, we make sure
        // we're never going to forget to reset anything specific.
        *self = Self::new(code);
    }

    pub fn update(&mut self, code: &Codebase) {
        if let Some(id) = &mut self.next {
            *id = code.latest_version_of(id);
        }

        for active_call in &mut self.active_calls {
            active_call.node = code.latest_version_of(&active_call.node);
        }
    }

    pub fn next(&self) -> Option<&NodeId> {
        self.next.as_ref()
    }

    pub fn state(&self, code: &Codebase) -> InterpreterState {
        use InterpreterState::*;

        match self.next_expression(code) {
            NextExpression::Expression { .. } => Running,
            NextExpression::NoMoreFragments if self.active_calls.is_empty() => {
                Finished
            }
            NextExpression::NoMoreFragments => Error,
            NextExpression::NextFragmentIsNotAnExpression => Error,
        }
    }

    pub fn step(&mut self, code: &Codebase) -> StepResult {
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
                    if let Some(ActiveCall {
                        output: Some(output),
                        ..
                    }) = self.active_calls.last()
                    {
                        let output = output.clone();
                        self.active_calls.pop();
                        return self.evaluate_value(output);
                    }

                    self.active_calls.push(ActiveCall {
                        node: fragment,
                        output: None,
                        target: *target,
                    });
                    self.next = body.entry().copied();
                }
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
        let Some(ActiveCall {
            node: fragment,
            output,
            target,
        }) = self.active_calls.last_mut()
        else {
            self.next = None;
            return StepResult::Finished { output: value };
        };

        self.next = Some(*fragment);

        match target {
            FunctionCallTarget::HostFunction { id } => {
                StepResult::CallToHostFunction {
                    id: *id,
                    input: value,
                    output: output.insert(Value::Integer { value: 0 }),
                }
            }
            FunctionCallTarget::IntrinsicFunction => {
                *output = Some(value);
                StepResult::CallToIntrinsicFunction
            }
        }
    }

    pub fn next_expression<'r>(
        &self,
        code: &'r Codebase,
    ) -> NextExpression<'r> {
        let Some(id) = self.next else {
            return NextExpression::NoMoreFragments;
        };
        let node = code.nodes().get(&id);
        let FragmentKind::Expression { expression } = &node.kind else {
            return NextExpression::NextFragmentIsNotAnExpression;
        };

        NextExpression::Expression {
            expression,
            body: &node.body,
            fragment: id,
        }
    }
}

#[derive(Debug)]
struct ActiveCall {
    node: NodeId,
    output: Option<Value>,
    target: FunctionCallTarget,
}

#[derive(Debug, Eq, PartialEq)]
pub enum InterpreterState {
    Running,
    Finished,
    Error,
}

impl InterpreterState {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    #[cfg(test)]
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished)
    }
}

#[derive(Debug, PartialEq)]
pub enum StepResult<'r> {
    CallToHostFunction {
        id: usize,
        input: Value,
        output: &'r mut Value,
    },
    CallToIntrinsicFunction,
    Error,
    Finished {
        output: Value,
    },
}

#[derive(Debug)]
pub enum NextExpression<'r> {
    Expression {
        expression: &'r Expression,
        body: &'r Body,
        fragment: NodeId,
    },
    NoMoreFragments,
    NextFragmentIsNotAnExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer { value: u32 },
}
