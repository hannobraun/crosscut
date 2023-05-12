use std::collections::BTreeMap;

use super::syntax::SyntaxTree;

pub struct Functions {
    functions: BTreeMap<String, SyntaxTree>,
}

impl Functions {
    pub fn new() -> Self {
        Self {
            functions: BTreeMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<SyntaxTree> {
        self.functions.get(name).cloned()
    }

    pub fn define_fn(&mut self, name: String, body: SyntaxTree) {
        self.functions.insert(name, body);
    }
}
