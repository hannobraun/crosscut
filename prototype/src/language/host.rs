use std::collections::BTreeMap;

use super::code::HostFunction;

pub struct Host {
    pub functions: BTreeMap<String, HostFunction>,
}

impl Host {
    pub fn functions(
        &self,
    ) -> impl Iterator<Item = (&String, HostFunction)> + '_ {
        self.functions
            .iter()
            .map(|(name, function)| (name, *function))
    }

    pub fn function_by_name(&self, name: &str) -> Option<HostFunction> {
        self.functions()
            .find_map(|(n, function)| (n == name).then_some(function))
    }
}
