use std::collections::BTreeMap;

use crate::syntax::{Clusters, Expression, IdentifierTarget};

pub fn mark_recursive_calls(clusters: &mut Clusters) {
    for cluster in &mut clusters.clusters {
        let indices_in_cluster_by_function_name = cluster
            .functions
            .iter()
            .filter_map(|(&function_index_in_cluster, named_function_index)| {
                clusters.functions[named_function_index]
                    .name
                    .clone()
                    .map(|name| (name, function_index_in_cluster))
            })
            .collect::<BTreeMap<_, _>>();

        for named_function_index in cluster.functions.values() {
            let function = clusters
                .functions
                .get_mut(named_function_index)
                .expect("Functions referred to from clusters must exist.");

            for branch in &mut function.branches {
                for expression in &mut branch.body {
                    if let Expression::Identifier {
                        name,
                        target:
                            Some(IdentifierTarget::Function {
                                is_known_to_be_recursive_call_to_index,
                            }),
                        ..
                    } = expression
                    {
                        if let Some(&index) =
                            indices_in_cluster_by_function_name.get(name)
                        {
                            *is_known_to_be_recursive_call_to_index =
                                Some(index);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        host::NoHost,
        passes::{group_into_clusters, parse, resolve_identifiers, tokenize},
        syntax::{Clusters, Expression, IdentifierTarget},
    };

    #[test]
    fn self_recursive_functions() {
        let clusters = mark_recursive_calls(
            r"
                f: {
                    \ ->
                        f
                }

                # Need a second function in this test, because for the first,
                # the index of the function within the list of all named
                # functions and the index of the function within its cluster,
                # are both `0`.
                #
                # This is prone to hide a bug, so we have this second function,
                # that has a non-zero index in the list of all named functions.
                g: {
                    \ ->
                        g
                }
            ",
        );

        for mut function in clusters.functions.into_values() {
            let Expression::Identifier { target, .. } =
                function.branches.remove(0).body.remove(0)
            else {
                panic!("Expected expression to be an identifier.");
            };
            let Some(IdentifierTarget::Function {
                is_known_to_be_recursive_call_to_index,
            }) = target
            else {
                panic!("Expected expression to be a function call");
            };
            let Some(index) = is_known_to_be_recursive_call_to_index else {
                panic!("Expected function call to be recursive.");
            };

            assert_eq!(
                index.0, 0,
                "Function is only self-recursive, not mutually recursive. \
                Expecting it to be alone in a cluster, hence index referring \
                to it must be `0`."
            );
        }
    }

    fn mark_recursive_calls(source: &str) -> Clusters {
        let tokens = tokenize(source);
        let mut functions = parse(tokens);
        resolve_identifiers::<NoHost>(&mut functions);
        let mut clusters = group_into_clusters(functions);
        super::mark_recursive_calls(&mut clusters);
        clusters
    }
}