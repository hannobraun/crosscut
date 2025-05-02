use itertools::Itertools;

use crate::language::{
    code::NodePath,
    tests::infra::{LocatedNodeExt, expression},
};

use super::Codebase;

#[test]
fn uniquely_identify_identical_children_of_different_parents() {
    // Nodes that are identical, but have different parents, should be uniquely
    // identified.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let a = change_set.nodes.insert(expression("a", []));

        let parent_a = change_set.nodes.insert(expression("b", [a]));
        let c = change_set.nodes.insert(expression("c", [a]));

        let root = change_set.nodes.insert(expression("root", [parent_a, c]));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(root),
        );
    });

    let [a1, a2] = codebase
        .node_at(&codebase.root().path)
        .children(codebase.nodes())
        .map(|b_or_c| {
            let [a] = b_or_c.expect_children(codebase.nodes());
            a
        })
        .collect_array()
        .unwrap();

    assert_ne!(a1, a2);
}

#[test]
fn uniquely_identify_identical_siblings() {
    // Nodes that are identical siblings should be uniquely identified.

    let mut codebase = Codebase::new();

    let root = codebase.make_change(|change_set| {
        let a = change_set.nodes.insert(expression("a", []));
        let b = change_set.nodes.insert(expression("b", [a, a]));

        let b = NodePath::for_root(b);
        change_set.replace(&change_set.root_before_change(), &b);

        b
    });

    let [a1, a2] = codebase.node_at(&root).expect_children(codebase.nodes());

    assert_ne!(a1, a2);
}
