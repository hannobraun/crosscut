#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod effects;
mod evaluator;
mod function;
mod heap;
mod instructions;
mod operands;
mod runtime;
mod stack;
mod value;

pub use self::{
    effects::{Effect, TriggerResult, TriggeredEffect},
    function::{Branch, Function, Pattern},
    heap::Heap,
    instructions::{Instruction, InstructionAddress, Instructions},
    operands::{Operands, PopOperandError},
    runtime::{Runtime, RuntimeState},
    stack::Stack,
    value::Value,
};
