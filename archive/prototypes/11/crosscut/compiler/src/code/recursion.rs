use std::collections::BTreeSet;

use super::{
    syntax::{Expression, FunctionLocation, MemberLocation, SyntaxTree},
    Dependencies, FunctionCalls,
};

/// # Tracks recursive expressions
///
/// There are two types of expressions that can be recursive:
///
/// - Calls to user-defined functions
/// - Local functions
///
/// A call to a user-defined function can be recursive in the following ways:
///
/// - They are calling the named function in which they are defined. This means
///   they could be defined in that named function directly, or indirectly in
///   any of its local child functions, or any of their children, recursively.
/// - They are calling another function that call the named function in which
///   they are defined. The call from that other function could be indirect,
///   through any number of mutually recursive functions.
///
/// Local functions can be recursive in the following ways:
///
/// - They contain a recursive call.
/// - They contain a recursive function.
///
/// This means all recursion is rooted in a recursive function call, but that
/// recursive call "infects" any local function that contains it.
///
/// ## Implementation Note
///
/// As of this writing, recursive local functions are not tracked yet. This is
/// subject to an ongoing transition.
#[derive(Debug)]
pub struct Recursion {
    inner: BTreeSet<MemberLocation>,
}

impl Recursion {
    /// # Find all recursive expressions
    pub fn find(
        syntax_tree: &SyntaxTree,
        function_calls: &FunctionCalls,
        dependencies: &Dependencies,
    ) -> Self {
        let mut recursive_expressions = BTreeSet::new();

        for cluster in dependencies.clusters() {
            for function in cluster.functions(syntax_tree) {
                for branch in function.branches() {
                    for expression in branch.expressions() {
                        match expression.fragment {
                            Expression::Identifier { name: _ } => {
                                let Some(location) = function_calls
                                    .is_call_to_user_defined_function(
                                        &expression.location,
                                    )
                                else {
                                    continue;
                                };

                                if cluster.contains_function(location) {
                                    recursive_expressions
                                        .insert(expression.location);
                                }
                            }
                            Expression::LocalFunction { function: _ } => {
                                let location = FunctionLocation::from(
                                    expression.location.clone(),
                                );

                                if cluster.contains_function(&location) {
                                    recursive_expressions
                                        .insert(expression.location);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Self {
            inner: recursive_expressions,
        }
    }

    /// # Determine, if an expression is recursive
    pub fn is_recursive_expression(&self, location: &MemberLocation) -> bool {
        self.inner.contains(location)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        code::{syntax::SyntaxTree, Dependencies, FunctionCalls, Tokens},
        host::NoHost,
    };

    use super::Recursion;

    #[test]
    fn self_recursive_direct_call() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        nop
                        f
                    end
                end
            ",
        );

        let (nop, f) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&f));
    }

    #[test]
    fn self_recursive_indirect_call() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        fn
                            br ->
                                nop
                                f
                            end
                        end
                    end
                end
            ",
        );

        let (nop, f) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .filter_map(|expression| expression.into_local_function())
            .next()
            .unwrap()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&f));
    }

    #[test]
    fn mutually_recursive_direct_call() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        nop
                        g
                    end
                end

                g: fn
                    br ->
                        f
                    end
                end
            ",
        );

        let (nop, g) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&g));
    }

    #[test]
    fn mutually_recursive_indirect_call() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        fn
                            br ->
                                nop
                                g
                            end
                        end
                    end
                end

                g: fn
                    br ->
                        f
                    end
                end
            ",
        );

        let (nop, g) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .filter_map(|expression| expression.into_local_function())
            .next()
            .unwrap()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&g));
    }

    #[test]
    fn self_recursive_direct_local_function() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        nop
                        fn
                            br ->
                                f
                            end
                        end
                    end
                end
            ",
        );

        let (nop, function) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&function));
    }

    #[test]
    fn self_recursive_indirect_local_function() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        nop
                        fn
                            br ->
                                fn
                                    br ->
                                        f
                                    end
                                end
                            end
                        end
                    end
                end
            ",
        );

        let (nop, function) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&function));
    }

    #[test]
    fn mutually_recursive_direct_local_function() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        nop
                        fn
                            br ->
                                g
                            end
                        end
                    end
                end

                g: fn
                    br ->
                        f
                    end
                end
            ",
        );

        let (nop, function) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&function));
    }

    #[test]
    fn mutually_recursive_indirect_local_function() {
        let (syntax_tree, recursion) = find_recursion(
            r"
                f: fn
                    br ->
                        nop
                        fn
                            br ->
                                fn
                                    br ->
                                        g
                                    end
                                end
                            end
                        end
                    end
                end

                g: fn
                    br ->
                        f
                    end
                end
            ",
        );

        let (nop, function) = syntax_tree
            .function_by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .expressions()
            .map(|expression| expression.location)
            .collect_tuple()
            .unwrap();

        assert!(!recursion.is_recursive_expression(&nop));
        assert!(recursion.is_recursive_expression(&function));
    }

    fn find_recursion(input: &str) -> (SyntaxTree, Recursion) {
        let tokens = Tokens::tokenize(input);
        let syntax_tree = SyntaxTree::parse(tokens);
        let function_calls = FunctionCalls::resolve(&syntax_tree, &NoHost);
        let dependencies = Dependencies::resolve(&syntax_tree, &function_calls);
        let recursion =
            Recursion::find(&syntax_tree, &function_calls, &dependencies);

        (syntax_tree, recursion)
    }
}
