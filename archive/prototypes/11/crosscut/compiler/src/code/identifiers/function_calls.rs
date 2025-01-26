use std::collections::BTreeMap;

use crate::{
    code::syntax::{Expression, FunctionLocation, MemberLocation, SyntaxTree},
    host::{Host, HostFunction},
    intrinsics::IntrinsicFunction,
};

/// # Tracks function calls
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FunctionCalls {
    to_host_functions: BTreeMap<MemberLocation, HostFunction>,
    to_intrinsic_functions: BTreeMap<MemberLocation, IntrinsicFunction>,
    to_user_defined_functions: BTreeMap<MemberLocation, FunctionLocation>,
}

impl FunctionCalls {
    /// # Resolve all function calls
    pub fn resolve(syntax_tree: &SyntaxTree, host: &impl Host) -> Self {
        let mut to_host_functions = BTreeMap::new();
        let mut to_intrinsic_functions = BTreeMap::new();
        let mut to_user_defined_functions = BTreeMap::new();

        for function in syntax_tree.all_functions() {
            for branch in function.branches() {
                for expression in branch.expressions() {
                    if let Expression::Identifier { name } = expression.fragment
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

                        if let Some(function) =
                            syntax_tree.function_by_name(name)
                        {
                            to_user_defined_functions.insert(
                                expression.location,
                                function.location(),
                            );
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
        location: &MemberLocation,
    ) -> Option<&HostFunction> {
        self.to_host_functions.get(location)
    }

    /// # Determine, if an expression is a call to an intrinsic function
    pub fn is_call_to_intrinsic_function(
        &self,
        location: &MemberLocation,
    ) -> Option<&IntrinsicFunction> {
        self.to_intrinsic_functions.get(location)
    }

    /// # Determine, if an expression is a call to a user-defined function
    pub fn is_call_to_user_defined_function(
        &self,
        location: &MemberLocation,
    ) -> Option<&FunctionLocation> {
        self.to_user_defined_functions.get(location)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        code::{syntax::SyntaxTree, Tokens},
        host::{Host, HostFunction},
    };

    use super::FunctionCalls;

    #[test]
    fn resolve_host_function() {
        // The host can provide functions. Calls to these host functions should
        // be resolved as such.

        let (syntax_tree, function_calls) = resolve_function_calls(
            r"
                f: fn
                    br ->
                        host_fn
                    end
                end
            ",
        );

        let host_fn = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        assert!(function_calls.is_call_to_host_function(&host_fn).is_some());
        assert!(function_calls
            .is_call_to_intrinsic_function(&host_fn)
            .is_none());
        assert!(function_calls
            .is_call_to_user_defined_function(&host_fn)
            .is_none());
    }

    #[test]
    fn resolve_intrinsic_function() {
        // The compiler provides intrinsic functions. Calls to these should be
        // resolved as such.

        let (syntax_tree, function_calls) = resolve_function_calls(
            r"
                f: fn
                    br ->
                        nop
                    end
                end
            ",
        );

        let nop = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        assert!(function_calls.is_call_to_host_function(&nop).is_none());
        assert!(function_calls.is_call_to_intrinsic_function(&nop).is_some());
        assert!(function_calls
            .is_call_to_user_defined_function(&nop)
            .is_none());
    }

    #[test]
    fn resolve_user_defined_function() {
        // If a function is defined in the code, it should be resolved.

        let (syntax_tree, function_calls) = resolve_function_calls(
            r"
                f: fn
                    br ->
                        user_fn
                    end
                end

                user_fn: fn
                    br ->
                    end
                end
            ",
        );

        let nop = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .next()
            .unwrap();

        assert!(function_calls.is_call_to_host_function(&nop).is_none());
        assert!(function_calls.is_call_to_intrinsic_function(&nop).is_none());
        assert!(function_calls
            .is_call_to_user_defined_function(&nop)
            .is_some());
    }

    fn resolve_function_calls(input: &str) -> (SyntaxTree, FunctionCalls) {
        let tokens = Tokens::tokenize(input);
        let syntax_tree = SyntaxTree::parse(tokens);
        let function_calls = FunctionCalls::resolve(&syntax_tree, &TestHost);

        (syntax_tree, function_calls)
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
