use super::code::{Code, Expression, Fragment, Hash};

pub struct Interpreter {
    pub next: Option<Hash>,
}

impl Interpreter {
    pub fn new(code: &Code) -> Self {
        Self { next: code.entry() }
    }

    pub fn state(&self, code: &Code) -> &'static str {
        match self.next_expression(code) {
            NextExpression::Expression { .. } => "running",
            NextExpression::NoMoreFragments => "finished",
            NextExpression::NextFragmentIsNotAnExpression => "error",
        }
    }

    pub fn step(&mut self, code: &Code) -> InterpreterState {
        let NextExpression::Expression { expression } =
            self.next_expression(code)
        else {
            return InterpreterState::Error;
        };

        match expression {
            Expression::FunctionCall {
                target: _,
                argument: _,
            } => {
                // Not yet implemented.
                todo!()
            }
            Expression::LiteralValue { value } => {
                self.next = None;
                InterpreterState::Finished { output: *value }
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
        let fragment = code.fragment_by_hash(&hash);
        let Fragment::Expression { expression } = fragment else {
            return NextExpression::NextFragmentIsNotAnExpression;
        };

        NextExpression::Expression { expression }
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpreterState {
    Error,
    Finished { output: u32 },
}

pub enum NextExpression<'r> {
    Expression { expression: &'r Expression },
    NoMoreFragments,
    NextFragmentIsNotAnExpression,
}
