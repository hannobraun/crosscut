use crate::language::{
    code::{Body, NodePath},
    tests::infra::{ExpectChildren, identifier},
};

use super::Codebase;

#[test]
fn uniquely_identify_identical_children_of_different_parents() {
    // Nodes that are identical, but have different parents, should be uniquely
    // identified.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent_a = Body::default()
            .with_children([identifier("child")])
            .into_syntax_node(change_set.nodes);
        let parent_b = Body::default()
            .with_children([identifier("child")])
            .into_syntax_node(change_set.nodes);

        let root = {
            let node = Body::default()
                .with_children([parent_a, parent_b])
                .into_syntax_node(change_set.nodes);

            change_set.nodes.insert(node)
        };

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(root),
        );
    });

    let [parent_a, parent_b, _] =
        codebase.root().expect_children(codebase.nodes());
    let [child_a, child_b] = [parent_a, parent_b].map(|parent| {
        let [child, _] = parent.expect_children(codebase.nodes());
        child
    });

    assert_ne!(child_a, child_b);
}

#[test]
fn uniquely_identify_identical_siblings() {
    // Nodes that are identical siblings should be uniquely identified.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent = {
            let node = Body::default()
                .with_children([identifier("child"), identifier("child")])
                .into_syntax_node(change_set.nodes);

            change_set.nodes.insert(node)
        };

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let [child_a, child_b, _] =
        codebase.root().expect_children(codebase.nodes());

    assert_ne!(child_a, child_b);
}
