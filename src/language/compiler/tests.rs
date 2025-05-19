use crate::language::{
    code::{Codebase, NodePath},
    compiler::{Compiler, Tuple},
    tests::infra::{ExpectChildren, expression, identifier},
};

#[test]
fn insert_child() {
    // The compiler can insert a child node.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent = {
            let node = Tuple::default().into_syntax_node(change_set.nodes);
            change_set.nodes.insert(node)
        };

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let mut compiler = Compiler::new(&mut codebase);

    let child = compiler.insert_child(compiler.codebase().root().path, "child");

    let [child_of_root, _] = compiler
        .codebase()
        .root()
        .expect_children(compiler.codebase().nodes());
    assert_eq!(child_of_root.path, child);
}

#[test]
fn insert_child_with_grandparent() {
    // Inserting a child still works, if that child's parent has a parent
    // itself.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent = Tuple::default().into_syntax_node(change_set.nodes);
        let grandparent = {
            let node = Tuple::default()
                .with_values([parent])
                .into_syntax_node(change_set.nodes);
            change_set.nodes.insert(node)
        };

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(grandparent),
        );
    });

    let mut compiler = Compiler::new(&mut codebase);

    let grandparent = compiler.codebase().root();
    let [parent, _] = grandparent.expect_children(compiler.codebase().nodes());
    let child = compiler.insert_child(parent.path, "child");

    let [child_of_root, _] = compiler
        .codebase()
        .root()
        .expect_children(compiler.codebase().nodes());
    let [grandchild_of_root, _] =
        child_of_root.expect_children(compiler.codebase().nodes());
    assert_eq!(grandchild_of_root.path, child);
}

#[test]
fn replace_second_of_two_equal_children() {
    // If two children are equal, and one is replaced, the replacement logic
    // should correctly distinguish between them.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent = {
            let node = Tuple::default()
                .with_values([identifier("child"), identifier("child")])
                .into_syntax_node(change_set.nodes);

            change_set.nodes.insert(node)
        };

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let [_, child, _] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut compiler = Compiler::new(&mut codebase);
    compiler.replace(&child, "updated");

    let [child, updated, _] = codebase.root().expect_children(codebase.nodes());

    assert_eq!(child.node, &identifier("child"));
    assert_eq!(updated.node, &identifier("updated"));
}

#[test]
fn updating_child_updates_parent() {
    // If the child of a parent node is being updated, the parent node should be
    // updated as well.

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let child = change_set.nodes.insert(identifier("old"));
        let parent = change_set.nodes.insert(expression("unresolved", [child]));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let mut compiler = Compiler::new(&mut codebase);

    let [child] = compiler
        .codebase()
        .root()
        .expect_children(compiler.codebase().nodes());
    let child = compiler.replace(&child.path, "new");

    // After editing the child, the new parent node should be the same as the
    // old one, but with an updated child.
    let parent = compiler.codebase().root().path;
    assert_eq!(
        codebase.nodes().get(parent.hash()).children(),
        vec![*child.hash()],
    );
}
