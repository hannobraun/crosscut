use crate::language::{
    code::NodePath,
    tests::infra::{ExpectChildren, expression},
};

use super::Codebase;

#[test]
fn uniquely_identify_identical_children_of_different_parents() {
    // Nodes that are identical, but have different parents, should be uniquely
    // identified.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let child = change_set.nodes.insert(expression("child", []));

        let parent_a = change_set.nodes.insert(expression("parent_a", [child]));
        let parent_b = change_set.nodes.insert(expression("parent_b", [child]));

        let root = change_set
            .nodes
            .insert(expression("root", [parent_a, parent_b]));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(root),
        );
    });

    let [parent_a, parent_b] =
        codebase.root().expect_children(codebase.nodes());
    let [child_a, child_b] = [parent_a, parent_b].map(|parent| {
        let [child] = parent.expect_children(codebase.nodes());
        child
    });

    assert_ne!(child_a, child_b);
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
