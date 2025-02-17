use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Package<T: Function> {
    next_id: FunctionId,
    functions_by_name: BTreeMap<String, (T, FunctionId)>,
}

impl<T: Function> Package<T> {
    pub fn new() -> Self {
        Self {
            next_id: FunctionId { id: 0 },
            functions_by_name: BTreeMap::new(),
        }
    }

    pub fn function(&mut self, function: T) {
        assert_eq!(
            Some(function.id()),
            T::from_id(function.id()).map(|function| function.id()),
            "Function must return an ID that converts back into the same \
            function.",
        );

        let id = self.next_id;
        self.next_id = FunctionId { id: id.id + 1 };
        assert_eq!(id, function.id());

        self.functions_by_name
            .insert(function.name().to_string(), (function, id));
    }

    pub fn resolver(&self) -> Resolver {
        Resolver {
            function_ids_by_name: self
                .functions_by_name
                .iter()
                .map(|(_, (function, id))| (function.name().to_string(), *id))
                .collect(),
            function_names_by_id: self
                .functions_by_name
                .iter()
                .map(|(_, (function, id))| (*id, function.name().to_string()))
                .collect(),
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

impl Function for () {
    fn from_id(_: FunctionId) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn id(&self) -> FunctionId {
        FunctionId { id: 0 }
    }

    fn name(&self) -> &str {
        ""
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct FunctionId {
    pub id: u32,
}
