use std::collections::BTreeMap;

use crate::{
    intrinsics,
    repr::eval::{
        fragments::{FragmentId, FragmentPayload, Fragments},
        value::ValuePayload,
    },
    value::Value,
    PlatformFunction,
};

use super::{
    IntrinsicFunction, NativeFunction, UserDefined, UserDefinedFunction,
};

#[derive(Debug)]
pub struct Namespace<C> {
    bindings: BTreeMap<String, Value>,
    native_functions: BTreeMap<String, NativeFunction<C>>,
    user_defined_functions: BTreeMap<String, UserDefinedFunction>,
}

impl<C> Namespace<C> {
    pub fn new() -> Self {
        let mut native_functions = BTreeMap::new();

        for (intrinsic, name) in intrinsics::all() {
            native_functions
                .insert(name.to_string(), NativeFunction::Intrinsic(intrinsic));
        }

        Self {
            bindings: BTreeMap::new(),
            native_functions,
            user_defined_functions: BTreeMap::new(),
        }
    }

    pub fn register_platform(
        &mut self,
        functions: impl IntoIterator<Item = (&'static str, PlatformFunction<C>)>,
    ) {
        for (name, function) in functions {
            self.native_functions
                .insert(name.into(), NativeFunction::Platform(function));
        }
    }

    pub fn user_defined(&mut self) -> UserDefined {
        UserDefined {
            bindings: &mut self.bindings,
            functions: &mut self.user_defined_functions,
        }
    }

    pub fn resolve(
        &self,
        name: &str,
    ) -> Result<NamespaceItem<C>, ResolveError> {
        let native_function =
            self.native_functions.get(name).map(|native| match native {
                NativeFunction::Intrinsic(function) => {
                    NamespaceItem::IntrinsicFunction(function)
                }
                NativeFunction::Platform(function) => {
                    NamespaceItem::PlatformFunction(function)
                }
            });
        let user_defined_function =
            self.user_defined_functions.get(name).map(|user_defined| {
                NamespaceItem::UserDefinedFunction(user_defined)
            });
        let binding = self
            .bindings
            .get(name)
            .map(|binding| NamespaceItem::Binding(binding.clone()));

        native_function
            .or(user_defined_function)
            .or(binding)
            .ok_or(ResolveError { name: name.into() })
    }

    pub fn replace(
        &mut self,
        old: FragmentId,
        new: FragmentId,
        fragments: &Fragments,
    ) {
        let mut renames = Vec::new();

        for (old_name, UserDefinedFunction { name, body, .. }) in
            self.user_defined_functions.iter_mut()
        {
            if name.fragment == Some(old) {
                let fragment = fragments.get(new);
                let FragmentPayload::Value(ValuePayload::Symbol(new_name)) =
                    &fragment.payload
                else {
                    // If the new fragment is not a symbol, then it's not
                    // supposed to be a function name. Not sure if we can
                    // handle this any smarter.
                    continue;
                };

                name.value = new_name.clone();
                name.fragment = Some(new);

                renames.push((old_name.clone(), new_name.clone()));
            }
            if body.start == old {
                body.start = new;
            }
        }

        for (old, new) in renames {
            let function = self.user_defined_functions.remove(&old).unwrap();
            self.user_defined_functions.insert(new, function);
        }
    }
}

impl<C> Default for Namespace<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum NamespaceItem<'r, C> {
    Binding(Value),
    IntrinsicFunction(&'r IntrinsicFunction),
    PlatformFunction(&'r PlatformFunction<C>),
    UserDefinedFunction(&'r UserDefinedFunction),
}

#[derive(Debug, thiserror::Error)]
#[error("Error resolving function `{name}`")]
pub struct ResolveError {
    pub name: String,
}
