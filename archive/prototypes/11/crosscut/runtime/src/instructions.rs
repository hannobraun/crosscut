use alloc::{collections::BTreeSet, string::String, vec::Vec};
use core::fmt;

use crate::{Branch, Effect, Function, Value};

/// # The instructions that the runtime executes
pub struct Instructions<'r> {
    pub inner: &'r [(InstructionAddress, Instruction)],
}

impl Instructions<'_> {
    pub fn get(&self, address: &InstructionAddress) -> Option<&Instruction> {
        let (stored_address, instruction) =
            self.inner.get(address.to_usize())?;
        assert_eq!(address, stored_address);
        Some(instruction)
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct InstructionAddress {
    pub index: u32,
}

impl InstructionAddress {
    pub fn previous(&self) -> Self {
        Self {
            index: self.index - 1,
        }
    }

    pub fn next(&self) -> Self {
        Self {
            index: self.index + 1,
        }
    }

    pub fn to_usize(self) -> usize {
        self.index
            .try_into()
            .expect("Expected `usize` to cover full range of `u32`")
    }
}

impl fmt::Display for InstructionAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.index.fmt(f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Instruction {
    /// # Add two signed 8-bit integers, triggering an error on overflow
    AddS8,

    /// # Add two signed 32-bit integers, triggering an error on overflow
    AddS32,

    /// # Add two unsigned 8-bit integers, triggering an error on overflow
    AddU8,

    /// # Add two unsigned 8-bit integers, wrapping on overflow
    AddU8Wrap,

    /// # Bind a value to a name
    ///
    /// ## Implementation Note
    ///
    /// This is one of those high-level instructions that I'd like to get rid
    /// of, eventually. There's no need to know about the names of values at
    /// runtime.
    ///
    /// The compiler should just keep track of all names and the respective
    /// value's locations on the stack at compile-time. This requires changes to
    /// the instruction set, either in the form of an instruction that can copy
    /// any value to the top of the stack, or new means to specify which values
    /// a given instruction targets.
    Bind {
        name: String,
    },

    /// # Push the value that is bound to the provided name to the stack
    ///
    /// ## Implementation Note
    ///
    /// This is one of those high-level instructions that I'd like to get rid
    /// of, eventually. There's no need to know about the names of values at
    /// runtime.
    ///
    /// The compiler should just keep track of all names and the respective
    /// value's locations on the stack at compile-time. This requires changes to
    /// the instruction set, either in the form of an instruction that can copy
    /// any value to the top of the stack, or new means to specify which values
    /// a given instruction targets.
    BindingEvaluate {
        name: String,
    },

    /// # Call a function, selecting the right branch via pattern matching
    ///
    /// ## Implementation Note
    ///
    /// This instruction is overly complex. I think ideally, there would be two
    /// simple instructions instead:
    ///
    /// - `PushFrame`, to push a new stack frame, and do nothing else with it.
    /// - `ReuseFrame`, to prepare the existing stack frame for reuse. (If
    ///   bindings were just regular values whose addresses were managed by the
    ///   compiler, instead of being a special thing handled by the stack at
    ///   runtime, then we probably wouldn't need this at all, and could reuse
    ///   any stack frame without preparation. That's a different story,
    ///   however.)
    ///
    /// Those would just take a single address as an argument, and jump there
    /// directly. (And maybe even pushing a frame and jumping to an instructions
    /// could and should be decoupled eventually.)
    ///
    /// Pattern matching could move into the branches themselves. The compiler
    /// could generate the necessary code as required, using the following
    /// approach:
    ///
    /// - If there are no literal patterns among the arguments, just generate
    ///   nothing.
    /// - If there are literal patterns, generate code that compares them
    ///   against the call stack.
    ///   - If all patterns match, remove the respective operands from the
    ///     stack and continue with the rest of the branch as normal. (Although
    ///     probably the next step would be the compiler-generated code that
    ///     handles the arguments.
    ///   - If the patterns don't match, leave the operands as-is and jump
    ///     directly to the next branch, which continues with its own pattern
    ///     matching.
    ///
    /// This would require some new instructions for doing the comparison and
    /// jumping to another address, but that seems like the right direction to
    /// go in anyway.
    CallFunction {
        callee: Function,
        is_tail_call: bool,
    },

    /// # Convert a signed 32-bit number to a signed 8-bit number
    ConvertS32ToS8,

    /// # Copy a value on the stack to the top of the stack
    ///
    /// The value to copy is identified by an offset from the top of the stack,
    /// which this instruction expects as an argument.
    Copy,

    /// # Divide two signed 32-bit integers
    DivS32,

    /// # Divide two unsigned 8-bit integers
    DivU8,

    /// # Drop a value
    Drop,

    /// # Compare two values for equality
    Eq,

    /// # Evaluate an anonymous function
    ///
    /// The top value on the stack is interpreted as the index of the anonymous
    /// function. If it doesn't identify an anonymous function (because the
    /// anonymous function was evaluated previously, or such an index never
    /// existed), an error is triggered.
    ///
    /// ## Implementation Note
    ///
    /// This instruction is too high-level, and it's partially redundant with
    /// other high-level instructions. The duplicated code in their
    /// implementations within `Evaluator` is supporting evidence of this.
    ///
    /// Like other instructions, it needs to be replaced by smaller, more
    /// low-level ones. This requires the compiler to become smarter. Which is
    /// the direction I'd like things to go into anyway, but it hasn't fully
    /// happened yet.
    Eval {
        is_tail_call: bool,
    },

    /// # Determine if the first of two signed 8-bit numbers is greater
    GreaterS8,

    /// # Determine if the first of two signed 32-bit numbers is greater
    GreaterS32,

    /// # Determine if the first of two unsigned 8-bit numbers is greater
    GreaterU8,

    /// # Logical and
    LogicalAnd,

    /// # Logical not
    LogicalNot,

    /// # Create an anonymous function
    ///
    /// ## Implementation Note
    ///
    /// Anonymous functions and their environment are currently allocated in a
    /// special map within the evaluator. This instruction is required to make
    /// that work.
    ///
    /// But this situation is undesirable. Anonymous functions should not be
    /// boxed. They should just be regular values, with everything needed to
    /// make them work allocated on the stack.
    ///
    /// Right now, this is not feasible, because the language is untyped and
    /// every value is a 32-bit word. Once we have static typing and the ability
    /// to create composite values, anonymous functions should be represented
    /// using those.
    ///
    /// When this is the case, special handling at runtime will no longer be
    /// required. Then all logic specific to anonymous functions can live in the
    /// compiler, and this instruction can be removed.
    MakeAnonymousFunction {
        branches: Vec<Branch>,
        environment: BTreeSet<String>,
    },

    /// # Multiply two signed 32-bit numbers, triggering an error on overflow
    MulS32,

    /// # Multiply two unsigned 8-bit numbers, wrapping on overflow
    MulU8Wrap,

    /// # Negate a signed 32-bit number
    NegS32,

    /// # Do nothing (no operation)
    Nop,

    Push {
        value: Value,
    },

    /// # Compute the remainder of the division of two signed 32-bit numbers
    RemainderS32,

    Return,

    /// # Subtract two signed 32-bit numbers, triggering an error on overflow
    SubS32,

    /// # Subtract two unsigned 8-bit numbers, triggering an error on overflow
    SubU8,

    /// # Subtract two unsigned 8-bit numbers, wrapping on overflow
    SubU8Wrap,

    /// Trigger an effect
    TriggerEffect {
        effect: Effect,
    },
}
