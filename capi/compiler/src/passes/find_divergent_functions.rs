use std::collections::{BTreeMap, BTreeSet};

use petgraph::{algo::condensation, visit::EdgeRef, Direction, Graph};

use crate::code::{CallGraph, Expression, NamedFunctions};

pub fn find_divergent_functions(
    named_functions: &NamedFunctions,
    call_graph: &mut CallGraph,
) {
    for cluster in call_graph.clusters_from_leaves_mut() {
        let mut branch_call_graph = Graph::new();
        let mut node_index_by_branch_location = BTreeMap::new();

        for function in cluster.functions(named_functions) {
            for branch in function.branches() {
                let location = branch.location().clone();

                let index = branch_call_graph.add_node(location.clone());
                node_index_by_branch_location.insert(location, index);
            }
        }

        for function in cluster.functions(named_functions) {
            for branch in function.branches() {
                for expression in branch.body() {
                    if let Expression::CallToUserDefinedFunctionRecursive {
                        index,
                        ..
                    } = expression.find
                    {
                        let called_function_index =
                            cluster.functions.get(&index).expect(
                                "Function referred to from recursive call must \
                                exist in same cluster.",
                            );
                        let called_function = named_functions
                            .find_by_index(called_function_index)
                            .expect(
                                "Function referred to from cluster must exist.",
                            );

                        for called_branch in called_function.branches() {
                            let from = node_index_by_branch_location
                                [branch.location()];
                            let to = node_index_by_branch_location
                                [called_branch.location()];

                            branch_call_graph.add_edge(from, to, ());
                        }
                    }
                }
            }
        }

        let make_acyclic = false;
        let branch_clusters = condensation(branch_call_graph, make_acyclic);

        let mut diverging_branches = BTreeSet::new();

        for index in branch_clusters.node_indices() {
            let branch_cluster = branch_clusters.node_weight(index).expect(
                "Just got index by iterating over branch; must refer to a \
                node.",
            );

            let has_outgoing_edges = branch_clusters
                .edges_directed(index, Direction::Outgoing)
                .count()
                > 0;
            let only_contains_calls_to_itself = branch_clusters
                .edges_directed(index, Direction::Outgoing)
                .all(|edge| edge.target() == index);

            if has_outgoing_edges && only_contains_calls_to_itself {
                diverging_branches.extend(branch_cluster.iter().cloned());
            }
        }

        let diverging_functions = cluster
            .functions(named_functions)
            .filter_map(|function| {
                let all_branches_are_diverging =
                    function.branches().all(|branch| {
                        diverging_branches.contains(branch.location())
                    });

                if all_branches_are_diverging {
                    Some(function.index())
                } else {
                    None
                }
            })
            .collect();

        cluster.divergent_functions = Some(diverging_functions);
    }
}