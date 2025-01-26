//! # A more structured code representation that still mirrors the raw input
//!
//! Compared to [tokens](super::tokens), the syntax representation is a lot more
//! structured. It is the second code representation that the compiler produces,
//! and the last one that still mirrors the input.
//!
//! The syntax representation is the last code representation in the compiler
//! that is intended to be displayed to the developer. Follow-up representations
//! are geared towards simplicity and ease of processing by compiler code. They
//! no longer contain any syntax sugar, which is still present in the syntax
//! representation
//!
//! ## Implementation Note
//!
//! As of this writing, there is an ongoing cleanup of the code representations
//! in the compiler. What the above documentation says might not be fully true
//! yet.

mod location;
mod parse;
mod repr;

pub use self::{
    location::{
        BranchLocation, FunctionLocation, Located, MemberLocation,
        ParameterLocation,
    },
    repr::{
        expression::Expression,
        function::{
            Binding, Branch, Comment, Function, Member, NamedFunction,
            Parameter,
        },
        syntax_tree::SyntaxTree,
        types::SyntaxType,
    },
};
