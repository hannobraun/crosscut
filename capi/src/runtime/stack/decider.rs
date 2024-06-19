use crate::runtime::{Function, Instruction, MissingOperand, Value};

use super::{Bindings, Event, State};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stack {
    state: State,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            state: State::default(),
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn push_frame(
        &mut self,
        function: Function,
    ) -> Result<(), PushFrameError> {
        const RECURSION_LIMIT: usize = 8;
        if self.state.num_frames() >= RECURSION_LIMIT {
            return Err(PushFrameError::Overflow);
        }

        if self.state.num_frames() == 0 {
            // If there's no calling frame, then there's no place to take
            // arguments from. Make sure that the function doesn't expect any.
            assert_eq!(function.arguments.len(), 0);
        }

        let mut arguments = Bindings::new();

        for argument in function.arguments.iter().rev() {
            let value = self.pop_operand()?;
            arguments.insert(argument.clone(), value);
        }

        self.emit_event(Event::PushFrame { function });

        for (name, value) in arguments {
            self.define_binding(name, value);
        }

        Ok(())
    }

    pub fn pop_frame(&mut self) -> Result<(), StackIsEmpty> {
        if self.state.num_frames() == 0 {
            return Err(StackIsEmpty);
        }

        let return_values = self
            .state()
            .operands()
            .expect("Just confirmed that stack is not empty")
            .values()
            .collect::<Vec<_>>();

        self.state.frames.pop();

        if let Some(new_top) = self.state.frames.last_mut() {
            for value in return_values {
                new_top.operands.push(value);
            }
        };

        Ok(())
    }

    pub fn define_binding(&mut self, name: String, value: impl Into<Value>) {
        let value = value.into();
        self.emit_event(Event::DefineBinding { name, value })
    }

    pub fn push_operand(&mut self, operand: impl Into<Value>) {
        let operand = operand.into();
        self.emit_event(Event::PushOperand { operand });
    }

    pub fn pop_operand(&mut self) -> Result<Value, MissingOperand> {
        let mut operand = Err(MissingOperand);
        self.emit_event(Event::PopOperand {
            operand: &mut operand,
        });
        operand
    }

    pub fn consume_next_instruction(&mut self) -> Option<Instruction> {
        loop {
            let frame = self.state.frames.last_mut()?;

            let Some(instruction) = frame.function.consume_next_instruction()
            else {
                self.pop_frame()
                    .expect("Just accessed frame; must be able to pop it");
                continue;
            };

            return Some(instruction);
        }
    }

    fn emit_event(&mut self, event: Event) {
        self.state.evolve(event);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PushFrameError {
    #[error(transparent)]
    MissingOperand(#[from] MissingOperand),

    #[error("Reached recursion limit")]
    Overflow,
}

#[derive(Debug)]
pub struct StackIsEmpty;
