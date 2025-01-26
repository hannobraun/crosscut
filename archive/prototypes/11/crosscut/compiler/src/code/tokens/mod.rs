//! # A basic code representation, mirroring the raw input
//!
//! Tokens almost fully mirror the raw text that the developer inputs, only
//! ignoring whitespace. They are the first semi-structured code representation
//! that the compiler produces, laying the groundwork for parsing.

mod token;
mod tokenize;
mod tokens;

pub use self::{
    token::{Keyword, Punctuator, Token},
    tokens::{NoMoreTokens, Tokens},
};
