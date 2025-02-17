use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Package<T: Function> {
    next_id: FunctionId,
    functions_by_id: BTreeMap<FunctionId, T>,
}

impl<T: Function> Package<T> {
    pub fn new() -> Self {
        Self {
            next_id: FunctionId { id: 0 },
            functions_by_id: BTreeMap::new(),
        }
    }

    pub fn with_function(mut self, function: T) -> Self {
        let id = self.next_id;
        self.next_id = FunctionId { id: id.id + 1 };

        self.functions_by_id.insert(id, function);

        self
    }

    pub fn function_by_id(&self, id: FunctionId) -> &T {
        let Some(function) = self.functions_by_id.get(&id) else {
            panic!(
                "This method expects to be passed only IDs that have been \
                generated by the same instance of `Package`."
            );
        };

        function
    }
}

#[derive(Debug)]
pub struct Packages {
    inner: Vec<RegisteredPackage>,
}

impl Packages {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn register_package<T: Function>(&mut self, package: &Package<T>) {
        let package = RegisteredPackage {
            function_ids_by_name: package
                .functions_by_id
                .iter()
                .map(|(id, function)| (function.name().to_string(), *id))
                .collect(),
            function_names_by_id: package
                .functions_by_id
                .iter()
                .map(|(id, function)| (*id, function.name().to_string()))
                .collect(),
        };

        self.inner.push(package);
    }

    pub fn resolve_function(&self, name: &str) -> Option<FunctionId> {
        self.inner
            .iter()
            .find_map(|package| package.function_ids_by_name.get(name).copied())
    }

    pub fn function_name_by_id(&self, id: &FunctionId) -> &str {
        let Some(name) = self
            .inner
            .iter()
            .find_map(|package| package.function_names_by_id.get(id))
        else {
            panic!("Expected function ID `{id:?}` to be valid.");
        };

        name
    }
}

#[derive(Debug)]
struct RegisteredPackage {
    function_ids_by_name: BTreeMap<String, FunctionId>,
    function_names_by_id: BTreeMap<FunctionId, String>,
}

pub trait Function: Copy + Ord {
    fn name(&self) -> &str;
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct FunctionId {
    pub id: u32,
}
