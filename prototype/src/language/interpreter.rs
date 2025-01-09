use super::code::{Code, Expression, Fragment};

pub struct Interpreter {
    pub next_fragment: Option<usize>,
}

impl Interpreter {
    pub fn new(next_fragment: usize) -> Self {
        Self {
            next_fragment: Some(next_fragment),
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
        let NextExpression::Expression { expression } =
            self.next_expression(code)
        else {
            return InterpreterState::Error;
        };

        match expression {
            Expression::FunctionCall { target: _ } => {
                // Not yet implemented.
                todo!()
            }
            Expression::LiteralValue { value } => {
                self.next_fragment = None;
                InterpreterState::Finished { output: *value }
            }
        }
    }

    pub fn next_expression<'r>(&self, code: &'r Code) -> NextExpression<'r> {
        let Some(index) = self.next_fragment else {
            return NextExpression::NoMoreFragments;
        };
        let Some(fragment) = code.fragment_at(index) else {
            return NextExpression::NoMoreFragments;
        };
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
