use std::collections::BTreeMap;

use crate::{
    intrinsics,
    repr::eval::{
        fragments::{FragmentId, FragmentPayload, Fragments},
        value::ValuePayload,
    },
    PlatformFunction,
};

use super::{
    IntrinsicFunction, NativeFunction, UserDefinedFunction,
    UserDefinedFunctions,
};

#[derive(Debug)]
pub struct Namespace<C> {
    native: BTreeMap<String, NativeFunction<C>>,
    user_defined: BTreeMap<String, UserDefinedFunction>,
}

impl<C> Namespace<C> {
    pub fn new() -> Self {
        let mut native = BTreeMap::new();

        for (intrinsic, name) in intrinsics::all() {
            native
                .insert(name.to_string(), NativeFunction::Intrinsic(intrinsic));
        }

        Self {
            native,
            user_defined: BTreeMap::new(),
        }
    }

    pub fn register_platform(
        &mut self,
        functions: impl IntoIterator<Item = (&'static str, PlatformFunction<C>)>,
    ) {
        for (name, function) in functions {
            self.native
                .insert(name.into(), NativeFunction::Platform(function));
        }
    }

    pub fn user_defined(&mut self) -> UserDefinedFunctions {
        UserDefinedFunctions {
            inner: &mut self.user_defined,
        }
    }

    pub fn resolve(&self, name: &str) -> Result<Function<C>, ResolveError> {
        let native = self.native.get(name).map(|native| match native {
            NativeFunction::Intrinsic(function) => {
                Function::Intrinsic(function)
            }
            NativeFunction::Platform(function) => Function::Platform(function),
        });
        let user_defined = self
            .user_defined
            .get(name)
            .map(|user_defined| Function::UserDefined(user_defined));

        native
            .or(user_defined)
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
            self.user_defined.iter_mut()
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
            let function = self.user_defined.remove(&old).unwrap();
            self.user_defined.insert(new, function);
        }
    }
}

impl<C> Default for Namespace<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum Function<'r, C> {
    Intrinsic(&'r IntrinsicFunction),
    Platform(&'r PlatformFunction<C>),
    UserDefined(&'r UserDefinedFunction),
}

#[derive(Debug, thiserror::Error)]
#[error("Error resolving function `{name}`")]
pub struct ResolveError {
    pub name: String,
}
