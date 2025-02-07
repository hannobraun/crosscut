use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Package {
    function_ids_by_name: BTreeMap<String, FunctionId>,
    function_names_by_id: BTreeMap<FunctionId, String>,
}

impl Package {
    pub fn new() -> Self {
        Self {
            function_ids_by_name: BTreeMap::new(),
            function_names_by_id: BTreeMap::new(),
        }
    }

    pub fn function(&mut self, id: FunctionId, name: impl Into<String>) {
        let name = name.into();

        self.function_ids_by_name.insert(name.clone(), id);
        self.function_names_by_id.insert(id, name);
    }

    pub fn resolve_function(&self, name: &str) -> Option<FunctionId> {
        self.function_ids_by_name.get(name).copied()
    }

    pub fn function_name_by_id(&self, id: &u32) -> &str {
        let Some(name) = self.function_names_by_id.get(&FunctionId { id: *id })
        else {
            panic!("Expected function ID `{id}` to be valid.");
        };

        name
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FunctionId {
    pub id: u32,
}
