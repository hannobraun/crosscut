use std::collections::BTreeMap;

use super::code::HostFunction;

pub struct Host {
    pub functions: BTreeMap<String, HostFunction>,
}

impl Host {
    pub fn function_by_name(&self, name: &str) -> Option<HostFunction> {
        self.functions.get(name).copied()
    }
}
