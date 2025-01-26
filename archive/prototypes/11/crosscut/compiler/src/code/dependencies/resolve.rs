use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    algo::{condensation, toposort},
    Graph,
};

use crate::code::{
    syntax::{BranchLocation, Expression, FunctionLocation, SyntaxTree},
    FunctionCalls,
};

pub fn resolve_function_dependencies(
    syntax_tree: &SyntaxTree,
    function_calls: &FunctionCalls,
) -> Vec<Vec<FunctionLocation>> {
    let mut dependency_graph = Graph::new();
    let mut graph_indices_by_function_location = BTreeMap::new();

    for function in syntax_tree.all_functions() {
        graph_indices_by_function_location
            .entry(function.location.clone())
            .or_insert_with(|| dependency_graph.add_node(function.location));
    }

    for function in syntax_tree.all_functions() {
        let depender = graph_indices_by_function_location[&function.location];

        for branch in function.branches() {
            for expression in branch.expressions() {
                let dependee = match expression.fragment {
                    Expression::Identifier { .. } => function_calls
                        .is_call_to_user_defined_function(&expression.location)
                        .cloned(),
                    _ => expression
                        .into_local_function()
                        .map(|function| function.location),
                };

                if let Some(dependee) = dependee {
                    let dependee =
                        graph_indices_by_function_location[&dependee];
                    dependency_graph.add_edge(depender, dependee, ());
                }
            }
        }
    }

    collect_dependency_clusters(dependency_graph)
}

pub fn resolve_branch_dependencies(
    functions: &[FunctionLocation],
    syntax_tree: &SyntaxTree,
    function_calls: &FunctionCalls,
) -> Vec<BranchLocation> {
    let functions = functions.iter().collect::<BTreeSet<_>>();

    let mut unsorted_branches = functions
        .iter()
        .flat_map(|location| {
            let Some(function) = syntax_tree.function_by_location(location)
            else {
                unreachable!(
                    "Expecting `Dependencies` to not pass invalid function \
                    locations."
                );
            };

            function.branches()
        })
        .collect::<Vec<_>>();

    let mut sorted_branches = Vec::new();
    let mut functions_resolved_by_sorted_branches = BTreeSet::new();

    loop {
        let num_unsorted_branches_before = unsorted_branches.len();

        let mut i = 0;
        while let Some(branch) = unsorted_branches.get(i) {
            let branch_has_no_unresolved_dependencies =
                branch.expressions().all(|expression| {
                    let dependee = match expression.fragment {
                        Expression::Identifier { .. } => {
                            let Some(dependee) = function_calls
                                .is_call_to_user_defined_function(
                                    &expression.location,
                                )
                            else {
                                // If the identifier is not a call to a
                                // function, it's not relevant to determining
                                // the dependencies of the branch.
                                return true;
                            };

                            dependee.clone()
                        }
                        Expression::LocalFunction { .. } => {
                            // A branch depends on any function defined within
                            // it.
                            FunctionLocation::Local {
                                location: expression.location,
                            }
                        }
                        Expression::LiteralNumber { .. } => {
                            // Literals don't refer to functions, which makes
                            // them irrelevant to determining the dependencies
                            // of the branch.
                            return true;
                        }
                    };

                    let dependency_is_outside_of_cluster =
                        !functions.contains(&dependee);
                    let dependency_is_already_resolved =
                        functions_resolved_by_sorted_branches
                            .contains(&dependee);

                    dependency_is_outside_of_cluster
                        || dependency_is_already_resolved
                });

            if branch_has_no_unresolved_dependencies {
                functions_resolved_by_sorted_branches
                    .insert(branch.location.parent.clone());

                let branch = unsorted_branches.swap_remove(i);
                sorted_branches.push(branch.location);
            } else {
                i += 1;
            }
        }

        if num_unsorted_branches_before == unsorted_branches.len() {
            // The rest of the branches are mutually dependent. We're done.
            sorted_branches.extend(
                unsorted_branches.into_iter().map(|branch| branch.location),
            );
            break;
        }
    }

    sorted_branches
}

fn collect_dependency_clusters<T>(
    dependency_graph: Graph<T, ()>,
) -> Vec<Vec<T>> {
    let make_acyclic = true;
    let mut clustered_graph = condensation(dependency_graph, make_acyclic);

    let clustered_and_sorted_graph = toposort(&clustered_graph, None).expect(
        "The previous operation should have made the call graph acyclic. \
        Hence, topologically sorting the graph should not fail.",
    );

    clustered_and_sorted_graph
        .into_iter()
        .map(move |graph_index| {
            clustered_graph.remove_node(graph_index).expect(
                "Each entry in the sorted version of the call graph must \
                correspond to exactly one node in the unsorted version. So \
                using every node from the sorted version once, to remove its \
                respective node in the unsorted version, should always work.",
            )
        })
        .collect()
}
