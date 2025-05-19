use crate::language::{
    code::NodePath,
    compiler::Tuple,
    tests::infra::{ExpectChildren, expression, identifier},
};

use super::Codebase;

#[test]
fn uniquely_identify_identical_children_of_different_parents() {
    // Nodes that are identical, but have different parents, should be uniquely
    // identified.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent_a = Tuple::default()
            .with_values([identifier("child")])
            .into_syntax_node(change_set.nodes);
        let parent_b = Tuple::default()
            .with_values([identifier("child")])
            .into_syntax_node(change_set.nodes);

        let root = {
            let node = Tuple::default()
                .with_values([parent_a, parent_b])
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
        let child = change_set.nodes.insert(identifier("a"));
        let parent = change_set.nodes.insert(expression("b", [child, child]));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let [a1, a2] = codebase.root().expect_children(codebase.nodes());

    assert_ne!(a1, a2);
}
