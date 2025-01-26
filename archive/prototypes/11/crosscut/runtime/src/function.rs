use alloc::{collections::BTreeMap, string::String, vec::Vec};

use crate::{InstructionAddress, Value};

/// # The runtime representation of a function
///
/// ## Implementation Note
///
/// The goal is to remove this type. It is only needed here because of overly
/// complex instructions that should be replaced themselves.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct Function {
    pub branches: Vec<Branch>,
    pub environment: BTreeMap<String, Value>,
}

/// # The runtime representation of a branch
///
/// ## Implementation Note
///
/// The goal is to remove this type. It is only needed here because of overly
/// complex instructions that should be replaced themselves.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Branch {
    pub parameters: Vec<Pattern>,
    pub start: InstructionAddress,
}

/// # A pattern in a function argument
///
/// ## Implementation Note
///
/// This duplicates the type of the same name in `crosscut-compiler`. This is
/// deliberate, as a shared type would have to live here (as `crosscut-compiler`
/// depends on this crate), but it doesn't belong here. The need for this type
/// here is temporary, while so much of pattern matching is still figured out at
/// runtime.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Pattern {
    Identifier { name: String },
    Literal { value: Value },
}
