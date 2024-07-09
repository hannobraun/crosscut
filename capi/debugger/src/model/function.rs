use capi_compiler::{repr::fragments, source_map::SourceMap};
use capi_process::Process;

use super::FragmentModel;

#[derive(Clone, Eq, PartialEq)]
pub struct Function {
    pub name: String,
    pub fragments: Vec<FragmentModel>,
}

impl Function {
    pub fn new(
        mut function: fragments::Function,
        source_map: &SourceMap,
        process: &Process,
    ) -> Self {
        let fragments = function
            .fragments
            .drain()
            .map(|fragment| FragmentModel::new(fragment, source_map, process))
            .collect();

        Self {
            name: function.name,
            fragments,
        }
    }
}
