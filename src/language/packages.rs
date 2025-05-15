use std::collections::{BTreeMap, btree_map};

#[derive(Debug, Default)]
pub struct Packages {
    inner: BTreeMap<PackageId, RegisteredPackage>,
    next_id: u32,
}

impl Packages {
    pub fn new_package<T>(&mut self, functions: impl IntoIterator<Item = T>)
    where
        T: Function,
    {
        let package_id = {
            let id = self.next_id;

            let Some(next_id) = self.next_id.checked_add(1) else {
                panic!("Reached maximum number of supported packages.");
            };
            self.next_id = next_id;

            PackageId { id }
        };

        let registered = {
            let btree_map::Entry::Vacant(entry) = self.inner.entry(package_id)
            else {
                unreachable!(
                    "Duplicate package IDs are not possible. Above, we \
                    increment the ID every time we create a new package, and \
                    we don't allow the ID to wrap."
                );
            };

            entry.insert(RegisteredPackage::default())
        };

        let mut functions_by_id = BTreeMap::new();

        for (id, function) in (0..).zip(functions) {
            let id = FunctionId {
                id,
                package: package_id,
            };

            registered
                .function_ids_by_name
                .insert(function.name().to_string(), id);
            registered
                .function_names_by_id
                .insert(id, function.name().to_string());

            functions_by_id.insert(id, function);
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

#[derive(Debug, Default)]
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
    package: PackageId,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct PackageId {
    id: u32,
}
