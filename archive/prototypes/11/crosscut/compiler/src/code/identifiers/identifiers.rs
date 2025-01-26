use std::collections::BTreeMap;

use crate::{
    code::syntax::{
        Expression, FunctionLocation, MemberLocation, ParameterLocation,
        SyntaxTree,
    },
    host::HostFunction,
    intrinsics::IntrinsicFunction,
};

use super::{Bindings, FunctionCalls};

/// # Tracks which targets identifiers have been resolved to
#[derive(Debug)]
pub struct Identifiers {
    targets: BTreeMap<MemberLocation, IdentifierTarget>,
}

impl Identifiers {
    /// # Resolve all bindings
    pub fn resolve(
        syntax_tree: &SyntaxTree,
        bindings: &Bindings,
        function_calls: &FunctionCalls,
    ) -> Self {
        let mut targets = BTreeMap::new();

        for function in syntax_tree.all_functions() {
            for branch in function.branches() {
                for expression in branch.expressions() {
                    if let Expression::Identifier { .. } = expression.fragment {
                        let binding = bindings.is_binding(&expression.location);
                        let host_function = function_calls
                            .is_call_to_host_function(&expression.location);
                        let intrinsic_function = function_calls
                            .is_call_to_intrinsic_function(
                                &expression.location,
                            );
                        let user_defined_function = function_calls
                            .is_call_to_user_defined_function(
                                &expression.location,
                            );

                        let target = match (
                            binding,
                            host_function,
                            intrinsic_function,
                            user_defined_function,
                        ) {
                            (Some(binding), None, None, None) => {
                                IdentifierTarget::Binding(binding.clone())
                            }
                            (None, Some(host_function), None, None) => {
                                IdentifierTarget::HostFunction(
                                    host_function.clone(),
                                )
                            }
                            (None, None, Some(intrinsic_function), None) => {
                                IdentifierTarget::IntrinsicFunction(
                                    *intrinsic_function,
                                )
                            }
                            (None, None, None, Some(user_defined_function)) => {
                                IdentifierTarget::UserDefinedFunction(
                                    user_defined_function.clone(),
                                )
                            }
                            _ => {
                                panic!(
                                    "Identifier resolved to multiple targets:\n\
                                    \n\
                                    Binding: {binding:?}\n\
                                    Host function: {host_function:?}\n\
                                    Intrinsic function: {intrinsic_function:?} \
                                    \n\
                                    User-defined function: \
                                    {user_defined_function:?}\n"
                                );
                            }
                        };

                        targets.insert(expression.location, target);
                    }
                }
            }
        }

        Self { targets }
    }

    pub fn is_resolved(
        &self,
        location: &MemberLocation,
    ) -> Option<&IdentifierTarget> {
        self.targets.get(location)
    }
}

/// # The target that an identifier resolves to
#[derive(Debug)]
pub enum IdentifierTarget {
    /// # The identifier resolves to a binding
    Binding(ParameterLocation),

    /// # The identifier resolves to a host function
    HostFunction(HostFunction),

    /// # The identifier resolves to an intrinsic function
    IntrinsicFunction(IntrinsicFunction),

    /// # The identifier resolves to a user-defined function
    UserDefinedFunction(FunctionLocation),
}
