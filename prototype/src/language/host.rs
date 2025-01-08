use std::collections::BTreeSet;

use super::code::HostFunction;

pub struct Host {
    pub functions: BTreeSet<String>,
}

impl Host {
    #[cfg(test)]
    pub fn without_functions() -> Self {
        Self {
            functions: BTreeSet::new(),
        }
    }

    pub fn from_function_names(
        names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            functions: names.into_iter().map(|name| name.into()).collect(),
        }
    }

    pub fn functions(
        &self,
    ) -> impl Iterator<Item = (&String, HostFunction)> + '_ {
        self.functions
            .iter()
            .enumerate()
            .map(|(id, name)| (name, HostFunction { id }))
    }

    pub fn function_by_name(&self, name: &str) -> Option<HostFunction> {
        self.functions()
            .find_map(|(n, function)| (n == name).then_some(function))
    }
}
