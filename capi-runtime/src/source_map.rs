use std::collections::BTreeMap;

use crate::{runtime, syntax};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct SourceMap {
    runtime_to_syntax: BTreeMap<runtime::Location, syntax::Location>,
    syntax_to_runtime: BTreeMap<syntax::Location, runtime::Location>,
}

impl SourceMap {
    pub fn define_mapping(
        &mut self,
        runtime: runtime::Location,
        syntax: syntax::Location,
    ) {
        self.runtime_to_syntax.insert(runtime, syntax.clone());
        self.syntax_to_runtime.insert(syntax, runtime);
    }

    pub fn runtime_to_syntax(
        &self,
        runtime: &runtime::Location,
    ) -> syntax::Location {
        self.runtime_to_syntax
            .get(runtime)
            .cloned()
            .expect("Expect every runtime location to map to a syntax location")
    }

    pub fn syntax_to_runtime(
        &self,
        syntax: &syntax::Location,
    ) -> runtime::Location {
        self.syntax_to_runtime
            .get(syntax)
            .cloned()
            .expect("Expect every syntax location to map to a runtime location")
    }
}
