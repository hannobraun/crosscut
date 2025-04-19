use std::collections::{BTreeMap, btree_map};

#[derive(Debug)]
pub struct Packages {
    inner: BTreeMap<PackageId, RegisteredPackage>,
    next_id: u32,
}

impl Packages {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn new_package<T>(&mut self) -> PackageBuilder<T> {
        let package_id = {
            let id = self.next_id;

            let Some(next_id) = self.next_id.checked_add(1) else {
                panic!("Reached maximum number of supported packages.");
            };
            self.next_id = next_id;

            PackageId { id }
        };

        let btree_map::Entry::Vacant(entry) = self.inner.entry(package_id)
        else {
            unreachable!(
                "Duplicate package IDs are not possible. Above, we increment \
                the ID every time we create a new package, and we don't allow \
                the ID to wrap."
            );
        };
        let registered = entry.insert(RegisteredPackage::default());

        PackageBuilder {
            functions_by_id: BTreeMap::new(),
            registered,
            next_id: 0,
            package: package_id,
        }
    }

    pub fn resolve_function(&self, name: &str) -> Option<FunctionId> {
        self.inner
            .values()
            .find_map(|package| package.function_ids_by_name.get(name).copied())
    }

    pub fn function_name_by_id(&self, id: &FunctionId) -> &str {
        let Some(package) = self.inner.get(&id.package) else {
            panic!("Expected package ID `{:?}` to be valid.", id.package);
        };
        let Some(name) = package.function_names_by_id.get(id) else {
            panic!("Expected function ID `{id:?}` to be valid.");
        };

        name
    }
}

pub struct PackageBuilder<'r, T> {
    registered: &'r mut RegisteredPackage,
    functions_by_id: BTreeMap<FunctionId, T>,
    next_id: u32,
    package: PackageId,
}

impl<T> PackageBuilder<'_, T> {
    pub fn add_functions(&mut self, functions: impl IntoIterator<Item = T>)
    where
        T: Function,
    {
        for function in functions {
            let id = FunctionId {
                id: self.next_id,
                package: self.package,
            };
            self.next_id += 1;

            self.registered
                .function_ids_by_name
                .insert(function.name().to_string(), id);
            self.registered
                .function_names_by_id
                .insert(id, function.name().to_string());

            self.functions_by_id.insert(id, function);
        }
    }

    pub fn build(self) -> Package<T> {
        Package {
            functions_by_id: self.functions_by_id,
        }
    }
}

#[derive(Debug, Default)]
struct RegisteredPackage {
    function_ids_by_name: BTreeMap<String, FunctionId>,
    function_names_by_id: BTreeMap<FunctionId, String>,
}

#[derive(Debug)]
pub struct Package<T> {
    functions_by_id: BTreeMap<FunctionId, T>,
}

impl<T> Package<T> {
    pub fn function_by_id(&self, id: &FunctionId) -> Option<&T> {
        self.functions_by_id.get(id)
    }
}

pub trait Function: Copy + Ord {
    fn name(&self) -> &str;
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct FunctionId {
    id: u32,
    package: PackageId,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct PackageId {
    id: u32,
}
