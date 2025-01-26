pub mod code;
pub mod host;
pub mod intrinsics;
pub mod source_map;

mod compiler;
mod instructions;
mod passes;

#[cfg(test)]
mod tests;

pub use self::{
    compiler::{Compiler, CompilerOutput},
    instructions::Instructions,
};
