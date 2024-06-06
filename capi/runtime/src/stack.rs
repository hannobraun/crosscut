use std::collections::BTreeMap;

use crate::{operands::PopOperandError, InstructionAddress, Value};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Stack {
    inner: Vec<StackElement>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            inner: vec![
                StackElement::StartMarker,
                StackElement::Bindings(Bindings::new()),
            ],
        }
    }

    /// # Determine wether any stack frames are left
    ///
    /// The stack starts out with an initial stack frame on initialization. If
    /// no more stack frames are left, this means that the process is finished.
    ///
    /// ## Implementation Note
    ///
    /// In principle, no stack frames being left does not mean that the stack is
    /// empty. The process could have left some (frame-less) return value, and a
    /// host might expect such a thing.
    ///
    /// But right now, this method checks whether the stack is completely empty.
    /// This is a bug, which is tracked here:
    /// <https://github.com/hannobraun/caterpillar/issues/44>
    pub fn no_frames_left(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn bindings(&self) -> Option<&Bindings> {
        self.inner.iter().rev().find_map(|element| match element {
            StackElement::Bindings(bindings) => Some(bindings),
            _ => None,
        })
    }

    pub fn bindings_mut(&mut self) -> Option<&mut Bindings> {
        self.inner
            .iter_mut()
            .rev()
            .find_map(|element| match element {
                StackElement::Bindings(bindings) => Some(bindings),
                _ => None,
            })
    }

    pub fn operands_in_current_stack_frame(
        &self,
    ) -> impl Iterator<Item = &Value> + '_ {
        self.inner
            .iter()
            .rev()
            .take_while(|element| {
                !matches!(element, StackElement::ReturnAddress(_))
            })
            .filter_map(|element| match element {
                StackElement::Operand(value) => Some(value),
                _ => None,
            })
    }

    pub fn return_addresses(
        &self,
    ) -> impl Iterator<Item = InstructionAddress> + '_ {
        self.inner.iter().filter_map(|frame| match frame {
            StackElement::ReturnAddress(address) => Some(*address),
            _ => None,
        })
    }

    pub fn push_frame(
        &mut self,
        return_address: InstructionAddress,
    ) -> Result<(), PushStackFrameError> {
        // Not a tail call. This means we need to create a new stack frame.
        // Let's first check if we can even do that.
        const STACK_LIMIT: usize = 32;
        if self.inner.len() >= STACK_LIMIT {
            return Err(PushStackFrameError::Overflow);
        }

        // All stack frames but the initial one (which this one can't be, as the
        // initial one is created with the stack), start with a return address.
        self.inner.push(StackElement::ReturnAddress(return_address));

        // And all stack frames need a map of bindings.
        self.inner.push(StackElement::Bindings(Bindings::new()));

        Ok(())
    }

    pub fn reuse_frame(&mut self) {
        // We are repurposing the existing stack frame.
        //
        // This means the element that marks the start of the stack frame,
        // either the start marker or a return address, can stay as they are.
        //
        // So can operands. Those that are function arguments, we already
        // removed. Those that remain are what the tail-calling function
        // returns, so they can remain and be returned when the stack frame is
        // eventually done.
        //
        // But we need to handle bindings.

        let bindings = self.bindings_mut().expect(
            "Until the process has finished, there is always a stack frame. \
            Either the initial one, or one that was pushed while the process \
            was running.\n\
            \n\
            A new stack frame is being pushed right now, hence there must be \
            an existing one, which means it must be possible to find bindings.",
        );

        // Any bindings that remain are no longer accessible, so let's remove
        // them.
        bindings.clear();
    }

    pub fn pop_frame(&mut self) -> Option<InstructionAddress> {
        let mut index = self.inner.len();

        loop {
            if index == 0 {
                break None;
            }

            index -= 1;

            match self.inner[index] {
                StackElement::Bindings(_) => {
                    self.inner.remove(index);
                }
                StackElement::ReturnAddress(address) => {
                    self.inner.remove(index);
                    break Some(address);
                }
                StackElement::StartMarker => {
                    self.inner.remove(index);
                    break None;
                }
                _ => {}
            }
        }
    }

    pub fn define_binding(&mut self, name: String, value: impl Into<Value>) {
        self.bindings_mut()
            .expect("Expected stack frame to exist")
            .insert(name, value.into());
    }

    pub fn push_operand(&mut self, operand: impl Into<Value>) {
        self.inner.push(StackElement::Operand(operand.into()));
    }

    pub fn pop_operand(&mut self) -> Result<Value, PopOperandError> {
        let mut index = self.inner.len();
        while index > 0 {
            index -= 1;

            if let StackElement::Operand(value) = self.inner[index] {
                self.inner.remove(index);
                return Ok(value);
            }
        }

        Err(PopOperandError::MissingOperand)
    }

    pub fn into_inner(self) -> Vec<StackElement> {
        self.inner
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

/// # The things that can be on the stack
///
/// ## Implementation Note
///
/// This is an enum, because right now a lot of things still happen at runtime,
/// and the stack logic needs to recognize what kind of element it's looking at
/// to make an informed decision.
///
/// Eventually, the compiler will grow smarter, and be able to figure out what
/// needs to happen with the stack at compile-time. At that point, we will no
/// longer need to track this kind of type information at runtime.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum StackElement {
    /// # The bindings in the current stack frame
    ///
    /// There should be one of these per stack frame, and it's expected that
    /// this is the first element after the return address.
    ///
    /// ## Implementation Note
    ///
    /// Having a map with the names and values of bindings at runtime is
    /// unnecessary. We need the value, sure, but the compiler can track the
    /// names and locations of bindings at compile-time.
    ///
    /// At some point it will learn to do that, and generate the right
    /// instructions to access them. Then we won't need to look up bindings by
    /// name at runtime.
    Bindings(Bindings),

    /// An operand
    Operand(Value),

    /// A return address
    ///
    /// This marks the beginning of a stack frame. It carries the address that
    /// the evaluator needs to jump back to, once it's done with the current
    /// stack frame.
    ReturnAddress(InstructionAddress),

    /// A marker to substitute the return address in the initial stack frame
    ///
    /// The initial stack frame needs no return address, so it has this marker.
    /// The reason we need it, is to know when the first stack frame is being
    /// dropped, which indicates that the process has finished.
    ///
    /// Without a start marker, when we pop a frame, we wouldn't be able to
    /// distinguish whether the process has finished, or if we just happen to
    /// have an empty stack because of tail call elimination, but should still
    /// continue running.
    StartMarker,
}

pub type Bindings = BTreeMap<String, Value>;

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    thiserror::Error,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum PushStackFrameError {
    #[error(transparent)]
    PopOperand(#[from] PopOperandError),

    #[error("Reached recursion limit")]
    Overflow,

    #[error("Evaluator is already finished")]
    Finished,
}