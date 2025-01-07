use std::collections::BTreeMap;

use super::code::FunctionType;

pub struct Host {
    pub functions: BTreeMap<String, FunctionType>,
}
