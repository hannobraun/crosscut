use std::{collections::BTreeMap, iter};

use crate::{
    fragments::{
        Branch, BranchIndex, Fragment, FragmentId, FragmentIndexInBranchBody,
        FragmentMap, Fragments, Function, Parameters,
    },
    hash::{Hash, NextNeighbor, PrevNeighbor},
    syntax::{self, IdentifierTarget},
};

pub fn generate_fragments(clusters: syntax::Clusters) -> Fragments {
    let mut fragments = FragmentMap::default();

    let functions = clusters
        .clusters
        .iter()
        .rev()
        .flat_map(|cluster| cluster.functions.values())
        .map(|&index| {
            let function = clusters.functions[&index].clone();
            let fragment = compile_function(function, &mut fragments);
            (index, fragment)
        })
        .collect::<BTreeMap<_, _>>();

    let mut function_ids = Vec::new();
    let root = address_context(
        &functions
            .values()
            .cloned()
            .map(|function| Fragment::Function { function })
            .collect(),
        &mut function_ids,
        &mut fragments,
    );

    Fragments {
        root,
        functions,
        clusters: clusters.clusters,
    }
}

fn compile_function(
    function: syntax::Function,
    fragments: &mut FragmentMap,
) -> Function {
    let mut branches = Vec::new();

    for branch in function.branches {
        let body = branch
            .body
            .into_iter()
            .map(|expression| compile_expression(expression, fragments))
            .collect::<Vec<_>>();

        let body = iter::successors(Some(0), |i| Some(i + 1))
            .map(FragmentIndexInBranchBody)
            .zip(body)
            .collect();

        branches.push(Branch {
            parameters: Parameters {
                inner: branch.parameters,
            },
            body,
        });
    }

    let branches = iter::successors(Some(0), |i| Some(i + 1))
        .map(BranchIndex)
        .zip(branches)
        .collect();

    Function {
        name: function.name,
        branches,
        environment: function.environment,
        index_in_cluster: function.index_in_cluster,
    }
}

fn address_context(
    context: &Vec<Fragment>,
    ids: &mut Vec<FragmentId>,
    fragments: &mut FragmentMap,
) -> Option<FragmentId> {
    for fragment in context {
        ids.push(FragmentId {
            prev: None,
            next: None,
            content: Hash::new(fragment),
        });
    }

    let mut prev = None;

    for id in ids.iter_mut() {
        let prev_hash = prev.as_ref().map(Hash::new);

        id.prev = prev_hash;
        prev = Some(PrevNeighbor {
            ulterior_neighbor: prev_hash,
            content: id.content,
        });
    }

    let mut next = None;

    for id in ids.iter_mut().rev() {
        let next_hash = next.as_ref().map(Hash::new);

        id.next = next_hash;
        next = Some(NextNeighbor {
            ulterior_neighbor: next_hash,
            content: id.content,
        });
    }

    for (fragment, id) in context.iter().zip(&*ids) {
        fragments.insert(*id, fragment.clone());
    }

    ids.first().copied()
}

fn compile_expression(
    expression: syntax::Expression,
    fragments: &mut FragmentMap,
) -> Fragment {
    match expression {
        syntax::Expression::Comment { text } => Fragment::Comment { text },
        syntax::Expression::Function { function } => {
            let function = compile_function(function, fragments);
            Fragment::Function { function }
        }
        syntax::Expression::Identifier {
            name,
            target,
            is_known_to_be_in_tail_position,
        } => {
            // By the time we make it to this compiler pass, all expressions
            // that are in tail position should be known to be so.
            let is_in_tail_position = is_known_to_be_in_tail_position;

            match target {
                Some(IdentifierTarget::Binding) => {
                    Fragment::ResolvedBinding { name }
                }
                Some(IdentifierTarget::Function {
                    is_known_to_be_recursive_call_to_index,
                }) => {
                    // By the time we make it to this compiler pass, all calls
                    // that are recursive should be known to be so.
                    let is_recursive_call_to_index =
                        is_known_to_be_recursive_call_to_index;

                    if let Some(index) = is_recursive_call_to_index {
                        Fragment::CallToFunctionRecursive {
                            index,
                            is_tail_call: is_in_tail_position,
                        }
                    } else {
                        Fragment::CallToFunction {
                            name,
                            is_tail_call: is_in_tail_position,
                        }
                    }
                }
                Some(IdentifierTarget::HostFunction { effect_number }) => {
                    Fragment::CallToHostFunction { effect_number }
                }
                Some(IdentifierTarget::Intrinsic { intrinsic }) => {
                    Fragment::CallToIntrinsic {
                        intrinsic,
                        is_tail_call: is_in_tail_position,
                    }
                }
                None => Fragment::UnresolvedIdentifier { name },
            }
        }
        syntax::Expression::Value(value) => Fragment::Value(value),
    }
}
