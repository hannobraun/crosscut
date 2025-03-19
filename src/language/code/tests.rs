use itertools::Itertools;

use crate::language::code::NodePath;

use super::{Codebase, Node, NodeKind};

#[test]
fn uniquely_identify_identical_children_of_different_parents() {
    // Nodes that are identical, but have different parents, should be uniquely
    // identified.

    let mut codebase = Codebase::new();

    let [a, b, c, d] = ["a", "b", "c", "d"].map(|name| NodeKind::Error {
        node: name.to_string(),
    });

    let root = codebase.root().path;
    let root = codebase.make_change(|change_set| {
        let a = change_set.add(Node::new(a, []));
        let b = change_set.add(Node::new(b, [a]));
        let c = change_set.add(Node::new(c, [a]));
        let d = change_set.add(Node::new(d, [b, c]));

        let d = NodePath {
            hash: d,
            parent: None,
        };
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
