use super::code::{Code, Expression, Fragment};

#[derive(Default)]
pub struct Interpreter {
    pub next_fragment: usize,
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
        let Some(fragment) = self.next_fragment(code) else {
            return InterpreterState::Error;
        };

        match fragment {
            Expression::LiteralValue { value } => {
                // We increment the code pointer unconditionally, even if we
                // expect the program to be finished after this.
                //
                // This is important for two reasons:
                //
                // 1. If the program _is_ finished, then this fact can be
                //    derived from the interpreter state, even if a caller
                //    previously ignored the return value of this function.
                // 2. If the program is _not_ finished, then this is an
                //    error, and we want the next call to the `step`
                //    function to reflect that.
                self.next_fragment += 1;

                InterpreterState::Finished { output: *value }
            }
        }
    }

    pub fn next_fragment<'r>(&self, code: &'r Code) -> Option<&'r Expression> {
        let Fragment::Expression {
            expression: fragment,
        } = code.fragments.get(self.next_fragment)?
        else {
            return None;
        };

        Some(fragment)
    }
}

#[derive(Debug, PartialEq)]
pub enum InterpreterState {
    Error,

    Finished { output: f64 },
}
