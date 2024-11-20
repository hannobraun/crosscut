use std::collections::BTreeMap;

use crate::code::{
    Expression, Function, FunctionLocation, Functions, Index, Located,
    OrderedFunctions,
};

pub fn resolve_recursive_calls(
    functions: &mut Functions,
    ordered_functions: &OrderedFunctions,
) {
    for cluster in ordered_functions.clusters_from_leaves() {
        let indices_in_cluster_by_function_name = cluster
            .functions
            .iter()
            .filter_map(|(&function_index_in_cluster, function_location)| {
                let FunctionLocation::NamedFunction {
                    index: named_function_index,
                } = function_location
                else {
                    return None;
                };

                let name = functions
                    .named
                    .get(named_function_index)
                    .expect(
                        "Expecting function referenced from call graph to \
                        exist.",
                    )
                    .name
                    .clone();
                Some((name, function_index_in_cluster))
            })
            .collect::<BTreeMap<_, _>>();

        for function_location in cluster.functions.values() {
            let function = functions
                .by_location_mut(function_location)
                .expect("Functions referred to from clusters must exist.");

            resolve_recursive_calls_in_function(
                function,
                &indices_in_cluster_by_function_name,
            );
        }
    }
}

fn resolve_recursive_calls_in_function(
    function: Located<&mut Function>,
    indices_in_cluster_by_function_name: &BTreeMap<
        String,
        Index<FunctionLocation>,
    >,
) {
    let branches = function.destructure();

    for branch in branches {
        let (body, _) = branch.destructure();

        for expression in body {
            if let Expression::UnresolvedIdentifier {
                name,
                is_known_to_be_call_to_user_defined_function: true,
                ..
            } = expression.fragment
            {
                if let Some(&index) =
                    indices_in_cluster_by_function_name.get(name)
                {
                    *expression.fragment =
                        Expression::CallToUserDefinedFunctionRecursive {
                            index,
                        };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        code::{
            syntax::parse, Expression, FunctionLocation, Functions, Index,
            Tokens,
        },
        host::NoHost,
        passes::{order_functions_by_dependencies, resolve_most_identifiers},
    };

    #[test]
    fn self_recursive_functions() {
        let functions = resolve_recursive_calls(
            r"
                f: fn
                    \ ->
                        f
                end

                # Need a second function in this test, because for the first,
                # the index of the function within the list of all named
                # functions and the index of the function within its cluster,
                # are both `0`.
                #
                # This is prone to hide a bug, so we have this second function,
                # that has a non-zero index in the list of all named functions.
                g: fn
                    \ ->
                        g
                end
            ",
        );

        for mut function in functions.named.into_iter() {
            let Expression::CallToUserDefinedFunctionRecursive {
                index, ..
            } = function
                .inner
                .branches
                .pop_first()
                .map(|(_, branch)| branch)
                .unwrap()
                .body
                .pop_first()
                .map(|(_, expression)| expression)
                .unwrap()
            else {
                panic!("Expected expression to be an identifier.");
            };

            assert_eq!(
                index,
                Index::from(0),
                "Function is only self-recursive, not mutually recursive. \
                Expecting it to be alone in a cluster, hence index referring \
                to it must be zero."
            );
        }
    }

    #[test]
    fn mark_recursive_calls_from_anonymous_functions() {
        let functions = resolve_recursive_calls(
            r"
                f: fn
                    \ ->
                        fn
                            \ ->
                                f
                        end
                end
            ",
        );

        let f_a = functions
            .named
            .by_name("f")
            .unwrap()
            .into_located_function()
            .find_single_branch()
            .unwrap()
            .body()
            .next()
            .map(|expression| {
                let location =
                    FunctionLocation::from(expression.location.clone());
                functions.by_location(&location).unwrap()
            })
            .unwrap();
        let Expression::CallToUserDefinedFunctionRecursive { .. } = f_a
            .find_single_branch()
            .unwrap()
            .body()
            .next()
            .unwrap()
            .fragment
        else {
            panic!("Expected identifier to be a recursive function call.");
        };
    }

    fn resolve_recursive_calls(input: &str) -> Functions {
        let tokens = Tokens::from_input(input);
        let mut functions = parse(tokens);
        resolve_most_identifiers(&mut functions, &NoHost);
        let ordered_functions = order_functions_by_dependencies(&functions);
        super::resolve_recursive_calls(&mut functions, &ordered_functions);

        functions
    }
}
