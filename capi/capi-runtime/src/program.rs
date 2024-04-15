use crate::{Evaluator, Functions};

#[derive(Clone)]
pub struct Program {
    pub functions: Functions,
    pub evaluator: Evaluator,
    pub entry: usize,
}

impl Program {
    pub fn push(&mut self, arguments: impl IntoIterator<Item = usize>) {
        for value in arguments {
            self.evaluator.data_stack.push(value);
        }
    }

    pub fn run(&mut self, mem: &mut [u8]) {
        self.evaluator.instruction = self.entry;
        while self.evaluator.step(mem) {}
    }
}
