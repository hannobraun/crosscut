use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Host {
    function_ids_by_name: BTreeMap<String, u32>,
    function_names_by_id: BTreeMap<u32, String>,
}

impl Host {
    pub fn new() -> Self {
        Self {
            function_ids_by_name: BTreeMap::new(),
            function_names_by_id: BTreeMap::new(),
        }
    }

    pub fn function(&mut self, id: u32, name: impl Into<String>) {
        let name = name.into();

        self.function_ids_by_name.insert(name.clone(), id);
        self.function_names_by_id.insert(id, name);
    }

    pub fn function_id_by_name(&self, name: &str) -> Option<u32> {
        self.function_ids_by_name.get(name).copied()
    }

    pub fn function_name_by_id(&self, id: &u32) -> &str {
        let Some(name) = self.function_names_by_id.get(id) else {
            panic!("Expected function ID `{id}` to be valid.");
        };

        name
    }
}
