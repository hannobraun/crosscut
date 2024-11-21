use std::collections::{BTreeMap, BTreeSet};

use crate::{
    host::{Host, HostFunction},
    intrinsics::IntrinsicFunction,
};

use super::{Expression, ExpressionLocation, Functions};

/// # Tracks function calls
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FunctionCalls {
    to_host_functions: BTreeMap<ExpressionLocation, HostFunction>,
    to_intrinsic_functions: BTreeMap<ExpressionLocation, IntrinsicFunction>,
    to_user_defined_functions: BTreeSet<ExpressionLocation>,
}

impl FunctionCalls {
    /// # Resolve all function calls
    pub fn resolve(functions: &Functions, host: &impl Host) -> Self {
        let mut to_host_functions = BTreeMap::new();
        let mut to_intrinsic_functions = BTreeMap::new();
        let mut to_user_defined_functions = BTreeSet::new();

        for function in functions.all_functions() {
            for branch in function.branches() {
                for expression in branch.body() {
                    if let Expression::UnresolvedIdentifier { name } =
                        expression.fragment
                    {
                        // If multiple functions of different types have the
                        // same name, the following code will resolve a single
                        // identifier as multiple types of function call.
                        //
                        // This is by design. Later compiler passes can sort it
                        // out in whatever way they wish.

                        if let Some(function) = host.function_by_name(name) {
                            to_host_functions
                                .insert(expression.location.clone(), function);
                        }

                        if let Some(function) =
                            IntrinsicFunction::from_name(name)
                        {
                            to_intrinsic_functions
                                .insert(expression.location.clone(), function);
                        }

                        if functions.named.by_name(name).is_some() {
                            to_user_defined_functions
                                .insert(expression.location);
                        }
                    }
                }
            }
        }

        Self {
            to_host_functions,
            to_intrinsic_functions,
            to_user_defined_functions,
        }
    }

    /// # Determine, if an expression is a call to a host function
    pub fn is_call_to_host_function(
        &self,
        location: &ExpressionLocation,
    ) -> Option<&HostFunction> {
        self.to_host_functions.get(location)
    }

    /// # Determine, if an expression is a call to an intrinsic function
    pub fn is_call_to_intrinsic_function(
        &self,
        location: &ExpressionLocation,
    ) -> Option<&IntrinsicFunction> {
        self.to_intrinsic_functions.get(location)
    }

    /// # Determine, if an expression is a call to a user-defined function
    pub fn is_call_to_user_defined_function(
        &self,
        location: &ExpressionLocation,
    ) -> bool {
        self.to_user_defined_functions.contains(location)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        code::{syntax::parse, Functions, Tokens},
        host::{Host, HostFunction},
    };

    use super::FunctionCalls;

    #[test]
    fn resolve_host_function() {
        // The host can provide functions. Calls to these host functions should
        // be resolved as such.

        let (functions, function_calls) = resolve_function_calls(
            r"
                f: fn
                    \ ->
                        host_fn
                end
            ",
        );

        let host_fn = functions
            .named
            .by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .body()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        assert!(function_calls.is_call_to_host_function(&host_fn).is_some());
        assert!(function_calls
            .is_call_to_intrinsic_function(&host_fn)
            .is_none());
        assert!(!function_calls.is_call_to_user_defined_function(&host_fn));
    }

    #[test]
    fn resolve_intrinsic_function() {
        // The compiler provides intrinsic functions. Calls to these should be
        // resolved as such.

        let (functions, function_calls) = resolve_function_calls(
            r"
                f: fn
                    \ ->
                        nop
                end
            ",
        );

        let nop = functions
            .named
            .by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .body()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        assert!(function_calls.is_call_to_host_function(&nop).is_none());
        assert!(function_calls.is_call_to_intrinsic_function(&nop).is_some());
        assert!(!function_calls.is_call_to_user_defined_function(&nop));
    }

    #[test]
    fn resolve_user_defined_function() {
        // If a function is defined in the code, it should be resolved.

        let (functions, function_calls) = resolve_function_calls(
            r"
                f: fn
                    \ ->
                        user_fn
                end

                user_fn: fn
                    \ ->
                end
            ",
        );

        let nop = functions
            .named
            .by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .body()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        assert!(function_calls.is_call_to_host_function(&nop).is_none());
        assert!(function_calls.is_call_to_intrinsic_function(&nop).is_none());
        assert!(function_calls.is_call_to_user_defined_function(&nop));
    }

    fn resolve_function_calls(input: &str) -> (Functions, FunctionCalls) {
        let tokens = Tokens::from_input(input);
        let functions = parse(tokens);
        let function_calls = FunctionCalls::resolve(&functions, &TestHost);

        (functions, function_calls)
    }

    struct TestHost;

    impl Host for TestHost {
        fn functions(&self) -> impl IntoIterator<Item = HostFunction> {
            [HostFunction {
                name: "host_fn".into(),
                number: 0,
                signature: ([], []).into(),
            }]
        }
    }
}