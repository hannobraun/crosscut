use crate::language::{
    code::{CodeError, Codebase, NodeKind, NodePath},
    compiler::Compiler,
    packages::Packages,
};

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
        compiler.replace(compiler.codebase().root().path, "12", &packages);
    let parent = compiler.insert_parent(&child, "unresolved", &packages);

    // Verify our baseline assumptions about what the parent node should be.
    check_parent(parent, compiler.codebase());

    let child = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .next()
        .unwrap();
    compiler.replace(child.path, "127", &packages);

    // After editing the child, the new parent node should be the same as the
    // old one.
    let parent = compiler.codebase().root().path;
    check_parent(parent, &codebase);

    fn check_parent(parent: NodePath, codebase: &Codebase) {
        assert_eq!(
            codebase.node_at(parent).node.kind(),
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
