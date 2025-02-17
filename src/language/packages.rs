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
        assert_eq!(
            Some(function.id()),
            T::from_id(function.id()).map(|function| function.id()),
            "Function must return an ID that converts back into the same \
            function.",
        );

        self.function_ids_by_name
            .insert(function.name().to_string(), function.id());
        self.function_names_by_id
            .insert(function.id(), function.name().to_string());
    }

    pub fn resolver(&self) -> Resolver {
        Resolver {
            function_ids_by_name: self.function_ids_by_name.clone(),
            function_names_by_id: self.function_names_by_id.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Resolver {
    function_ids_by_name: BTreeMap<String, FunctionId>,
    function_names_by_id: BTreeMap<FunctionId, String>,
}

impl Resolver {
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

pub trait Function: Copy + Ord {
    fn from_id(id: FunctionId) -> Option<Self>
    where
        Self: Sized;

    fn from_verified_id(id: FunctionId) -> Self
    where
        Self: Sized,
    {
        let Some(function) = Self::from_id(id) else {
            panic!(
                "This function must already receive pre-verified function IDs \
                that result in a valid function."
            );
        };

        function
    }

    fn id(&self) -> FunctionId;
    fn name(&self) -> &str;
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct FunctionId {
    pub id: u32,
}
