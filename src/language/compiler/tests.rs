use itertools::Itertools;

use crate::language::{
    code::{CodeError, Codebase, NodeKind, NodePath},
    compiler::Compiler,
    packages::Packages,
};

#[test]
fn insert_child() {
    // The compiler can insert a child node.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let a =
        compiler.insert_child(compiler.codebase().root().path, "a", &packages);

    let [child_of_root] = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();
    assert_eq!(child_of_root, a);
}

#[test]
fn insert_child_with_grandparent() {
    // Inserting a child still works, if that child's parent has a parent
    // itself.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let a =
        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
    let b = compiler.insert_child(a.clone(), "b", &packages);

    let [child_of_root] = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .collect_array()
        .unwrap();
    let [grandchild_of_root] = child_of_root
        .children(compiler.codebase().nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();
    assert_eq!(grandchild_of_root, b);
}

#[test]
fn empty_node_with_multiple_children_is_an_error() {
    // An empty node has rather obvious runtime semantics: Do nothing and just
    // pass on the active value unchanged.
    //
    // If an empty node has multiple children, then it's no longer obvious what
    // it should do. So that needs to be an error.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    // Verify the assumptions this tests makes about the default root node.
    assert_eq!(compiler.codebase().root().node.kind(), &NodeKind::Empty);

    compiler.insert_child(compiler.codebase().root().path, "", &packages);
    compiler.insert_child(compiler.codebase().root().path, "", &packages);

    assert_eq!(
        compiler.codebase().root().node.kind(),
        &NodeKind::Error {
            node: "".to_string()
        },
    );

    let error = compiler.codebase().root().path;
    assert_eq!(
        compiler.codebase().errors().get(&error),
        Some(&CodeError::EmptyNodeWithMultipleChildren),
    );
}

#[test]
fn updating_child_updates_parent() {
    // If the child of a parent node is being updated, the parent node should be
    // updated as well.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let child =
        compiler.replace(&compiler.codebase().root().path, "12", &packages);
    let parent = compiler.insert_parent(&child, "unresolved", &packages);

    // Verify our baseline assumptions about what the parent node should be.
    check_parent(parent, compiler.codebase());

    let child = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .next()
        .unwrap();
    compiler.replace(&child.path, "127", &packages);

    // After editing the child, the new parent node should be the same as the
    // old one.
    let parent = compiler.codebase().root().path;
    check_parent(parent, &codebase);

    fn check_parent(parent: NodePath, codebase: &Codebase) {
        assert_eq!(
            codebase.node_at(&parent).node.kind(),
            &NodeKind::Error {
                node: "unresolved".to_string(),
            },
        );
        assert_eq!(
            codebase.errors().get(&parent),
            Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
        );
    }
}
