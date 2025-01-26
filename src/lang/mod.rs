//! Core components that are independent of a specific host and platform

pub mod code;
pub mod compiler;
pub mod editor;
pub mod host;
pub mod interpreter;

mod instance;

pub use self::instance::Language;

#[cfg(test)]
mod tests;
