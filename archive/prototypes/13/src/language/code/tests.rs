use itertools::Itertools;

use crate::language::{code::NodePath, tests::infra::node};

use super::Codebase;

#[test]
fn uniquely_identify_identical_children_of_different_parents() {
    // Nodes that are identical, but have different parents, should be uniquely
    // identified.

    let mut codebase = Codebase::new();

    let root = codebase.root().path;
    let root = codebase.make_change(|change_set| {
        let a = change_set.add(node("a", []));
        let b = change_set.add(node("b", [a]));
        let c = change_set.add(node("c", [a]));
        let d = change_set.add(node("d", [b, c]));

        let d = NodePath::for_root(d);
        change_set.replace(&root, &d);

        d
    });

    let (a1, a2) = codebase
        .node_at(&root)
        .children(codebase.nodes())
        .map(|b_or_c| {
            let [a] = b_or_c
                .children(codebase.nodes())
                .map(|located_node| located_node.path)
                .collect_array()
                .unwrap();

            a
        })
        .collect_tuple()
        .unwrap();

    assert_ne!(a1, a2);
}

#[test]
fn uniquely_identify_identical_siblings() {
    // Nodes that are identical siblings should be uniquely identified.

    let mut codebase = Codebase::new();

    let root = codebase.root().path;
    let root = codebase.make_change(|change_set| {
        let a = change_set.add(node("a", []));
        let b = change_set.add(node("b", [a, a]));

        let b = NodePath::for_root(b);
        change_set.replace(&root, &b);

        b
    });

    let [a1, a2] = codebase
        .node_at(&root)
        .children(codebase.nodes())
        .collect_array()
        .unwrap();

    assert_ne!(a1, a2);
}
