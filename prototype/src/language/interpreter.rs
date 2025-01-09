use super::code::{Code, Expression, Fragment};

pub struct Interpreter {
    pub next_fragment: usize,
}

impl Interpreter {
    pub fn new(next_fragment: usize) -> Self {
        Self { next_fragment }
    }

    pub fn state(&self, code: &Code) -> &'static str {
        match self.next_expression(code) {
            NextExpression::Expression { .. } => "running",
            NextExpression::NoMoreFragments => "finished",
            NextExpression::NextFragmentIsNotAnExpression => "error",
        }
    }

    pub fn step(&mut self, code: &Code) -> InterpreterState {
        let NextExpression::Expression { index, expression } =
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
                // We increment the code pointer unconditionally, even if we
                // expect the program to be finished after this.
                //
                // This is important for two reasons:
                //
                // 1. If the program _is_ finished, then this fact can be
                //    derived from the interpreter state, even if a caller
                //    previously ignored the return value of this function.
                // 2. If the program is _not_ finished, then this is an error,
                //    and we want the next call to the `step` function to
                //    reflect that.
                self.next_fragment = index + 1;

                InterpreterState::Finished { output: *value }
            }
        }
    }

    pub fn next_expression<'r>(&self, code: &'r Code) -> NextExpression<'r> {
        let index = self.next_fragment;
        let Some(fragment) = code.fragment_at(index) else {
            return NextExpression::NoMoreFragments;
        };
        let Fragment::Expression { expression } = fragment else {
            return NextExpression::NextFragmentIsNotAnExpression;
        };

        NextExpression::Expression { index, expression }
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpreterState {
    Error,
    Finished { output: u32 },
}

pub enum NextExpression<'r> {
    Expression {
        index: usize,
        expression: &'r Expression,
    },
    NoMoreFragments,
    NextFragmentIsNotAnExpression,
}
