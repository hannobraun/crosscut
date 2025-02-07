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

    pub fn function<T: Function>(&mut self, function: T) {
        self.function_ids_by_name
            .insert(function.name().to_string(), function.id());
        self.function_names_by_id
            .insert(function.id(), function.name().to_string());
    }

    pub fn resolve_function(&self, name: &str) -> Option<FunctionId> {
        self.function_ids_by_name.get(name).copied()
    }

    pub fn function_name_by_id(&self, id: &FunctionId) -> &str {
        let Some(name) = self.function_names_by_id.get(id) else {
            panic!("Expected function ID `{id:?}` to be valid.");
        };

        name
    }
}

pub trait Function {
    fn id(&self) -> FunctionId;
    fn name(&self) -> &str;
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct FunctionId {
    pub id: u32,
}
