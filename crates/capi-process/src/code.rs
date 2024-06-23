use std::collections::BTreeMap;

use crate::Function;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Code {
    pub functions: BTreeMap<String, Function>,
}
