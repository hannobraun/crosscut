use alloc::collections::BTreeMap;

use crate::Function;

/// # The heap memory used by the runtime
///
/// ## Implementation Note
///
/// The goal is to remove this completely and make the stack the only type of
/// memory used by the runtime. Right now, this can't be done, because closures
/// have to live somewhere, and they need to be boxed, for the time being.
///
/// Eventually, it will be possible to store closures unboxed on the stack. But
/// this requires a type system that supports values of different sizes, which
/// the current one (as of this writing) doesn't.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct Heap {
    pub(crate) closures: BTreeMap<u32, Function>,
    pub(crate) next_closure: u32,
}
