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

    pub fn function_id_by_name(&self, name: &str) -> Option<u32> {
        self.function_ids_by_name.get(name).copied()
    }

    pub fn function_name_by_id(&self, _id: &u32) -> &str {
        let Some(name) = self.function_names_by_id.get(_id) else {
            panic!("Expected function ID `{_id}` to be valid.");
        };

        name
    }
}
