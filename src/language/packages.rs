use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Package<T: Function> {
    functions_by_id: BTreeMap<FunctionId, T>,
    next_id: FunctionId,
}

impl<T: Function> Package<T> {
    pub fn new() -> Self {
        Self {
            functions_by_id: BTreeMap::new(),
            next_id: FunctionId { id: 0 },
        }
    }

    pub fn add_function(&mut self, function: T) -> FunctionId {
        let id = self.next_id;
        self.next_id = FunctionId { id: id.id + 1 };

        self.functions_by_id.insert(id, function);

        id
    }

    pub fn function_by_id(&self, id: &FunctionId) -> &T {
        let Some(function) = self.functions_by_id.get(id) else {
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
    next_id: PackageId,
}

impl Packages {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            next_id: PackageId { id: 0 },
        }
    }

    pub fn register_package<T: Function>(
        &mut self,
        package: &Package<T>,
    ) -> PackageId {
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

        let id = self.next_id;
        self.next_id.id += 1;

        id
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
    id: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct PackageId {
    pub id: u32,
}
