use std::collections::BTreeSet;

use super::code::{HostFunction, Signature};

pub struct Host {
    pub functions: BTreeSet<String>,
}

impl Host {
    pub fn functions(
        &self,
    ) -> impl Iterator<Item = (&String, HostFunction)> + '_ {
        self.functions.iter().enumerate().map(|(effect, name)| {
            (
                name,
                HostFunction {
                    id: effect,
                    signature: Signature {
                        input: (),
                        output: (),
                    },
                },
            )
        })
    }

    pub fn function_by_name(&self, name: &str) -> Option<HostFunction> {
        self.functions()
            .find_map(|(n, function)| (n == name).then_some(function))
    }
}
