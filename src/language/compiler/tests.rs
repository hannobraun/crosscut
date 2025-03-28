use itertools::Itertools;

use crate::language::{
    code::{Children, CodeError, Codebase, NodeHash, NodeKind, NodePath},
    compiler::Compiler,
    packages::{Function, Packages},
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
fn insert_child_should_update_errors() {
    // Inserting an erroneous child should insert an error into the codebase.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let unresolved = compiler.insert_child(
        compiler.codebase().root().path,
        "unresolved",
        &packages,
    );

    assert_eq!(
        compiler.codebase().errors().get(&unresolved),
        Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
    );
}

#[test]
fn remove_node_and_update_path_of_ancestor() {
    // Removing a node (like any change to a node) results in all of its
    // ancestors in the syntax tree being updated.
    //
    // So any existing `NodePath` that points to an ancestor of the removed node
    // and is required to refer to the current version of the same node after
    // the update, must be updated itself.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let parent =
        compiler.insert_child(compiler.codebase().root().path, "", &packages);
    let to_remove = compiler.insert_child(parent, "", &packages);

    let [to_update] = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();

    let mut updated = to_update.clone();
    compiler.remove(&to_remove, &mut updated, &packages);

    let root = compiler.codebase().root().path;
    assert!(!root.is_ancestor_of(&to_update));
    assert!(root.is_ancestor_of(&updated));
}

#[test]
fn remove_node_and_update_path_of_descendent() {
    // Removing a node (like any change to a node) results in all of its
    // ancestors in the syntax tree being updated, up to the root node.
    //
    // While descendants are not affected by this update, any `NodePath` that
    // refers to a descendant would no longer refer to the current version of
    // it, since the `NodePath` also includes a node's ancestors. And those have
    // changed, since the original update went up to the root node, which is the
    // ancestor of all nodes in the syntax tree.
    //
    // So any existing `NodePath` that points to a descendent of the removed
    // node and is required to refer to the current version of the same node
    // after the update, must be updated itself.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let parent =
        compiler.insert_child(compiler.codebase().root().path, "", &packages);
    let to_update = compiler.insert_child(parent, "", &packages);

    let [to_remove] = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();

    let mut updated = to_update.clone();
    compiler.remove(&to_remove, &mut updated, &packages);

    let root = compiler.codebase().root().path;
    assert!(!root.is_ancestor_of(&to_update));
    assert!(root.is_ancestor_of(&updated));
}

#[test]
fn remove_node_and_update_path_of_lateral_relation() {
    // Removing a node (like any change to a node) results in all of its
    // ancestors in the syntax tree being updated, up to the root node.
    //
    // While laterally related nodes (those that are neither ancestor nor
    // descendant, but share a common ancestor) are not affected by this update,
    // any `NodePath` that refers to such a node would no longer refer to the
    // current version of it, since the `NodePath` also includes a node's
    // ancestors. And those have changed, since the original update went up to
    // the root node, which is the ancestor of all nodes in the syntax tree.
    //
    // So any existing `NodePath` that points to a lateral relation of the
    // removed node and is required to refer to the current version of the same
    // node after the update, must be updated itself.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    compiler.insert_child(compiler.codebase().root().path, "a", &packages);
    compiler.insert_child(compiler.codebase().root().path, "b", &packages);

    let [to_remove, to_update] = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();

    let mut updated = to_update.clone();
    compiler.remove(&to_remove, &mut updated, &packages);

    let root = compiler.codebase().root().path;
    assert!(!root.is_ancestor_of(&to_update));
    assert!(root.is_ancestor_of(&updated));
}

#[test]
fn remove_node_and_update_path_of_sibling() {
    // Removing a node might result in the indices of its siblings being
    // affected. So any existing `NodePath` that points to a sibling of the
    // removed node and is required to refer to the current version of the same
    // node after the update, must be updated itself.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    compiler.insert_child(compiler.codebase().root().path, "", &packages);
    compiler.insert_child(compiler.codebase().root().path, "", &packages);

    let [to_remove, to_update] = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();

    let mut updated = to_update.clone();
    compiler.remove(&to_remove, &mut updated, &packages);

    assert_eq!(to_update.sibling_index(), 1);
    assert_eq!(updated.sibling_index(), 0);
}

#[test]
fn empty_node_with_multiple_children_is_an_error() {
    // An empty node has rather obvious runtime semantics: Do nothing and just
    // pass on the active value unchanged.
    //
    // If an empty node has multiple children, then it's no longer obvious what
    // it should do. So that needs to be an error.

    let packages = Packages::new();

    expect_error_on_multiple_children("", &packages);
}

#[test]
fn provided_function_application_with_multiple_children_is_an_error() {
    // A provided function application can only have one child: Its argument.

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Provided;
    impl Function for Provided {
        fn name(&self) -> &str {
            "provided"
        }
    }

    let mut packages = Packages::new();
    packages.new_package().add_function(Provided);

    expect_error_on_multiple_children("provided", &packages);
}

#[test]
fn integer_literal_with_children_is_an_error() {
    // An integer literal already carries all of the information that it needs
    // to evaluate to an integer. There is nothing it could do with children,
    // except ignore them.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    compiler.replace(&compiler.codebase().root().path, "127", &packages);
    let child =
        compiler.insert_child(compiler.codebase().root().path, "", &packages);

    let root = compiler.codebase().root();
    assert_eq!(
        root.node.kind(),
        &NodeKind::Error {
            node: "127".to_string(),
            children: Children::new([*child.hash()])
        },
    );
    assert_eq!(
        compiler.codebase().errors().get(&root.path),
        Some(&CodeError::IntegerLiteralWithChildren),
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
    check_parent(parent, [*child.hash()], compiler.codebase());

    let child = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .next()
        .unwrap();
    let child = compiler.replace(&child.path, "127", &packages);

    // After editing the child, the new parent node should be the same as the
    // old one.
    let parent = compiler.codebase().root().path;
    check_parent(parent, [*child.hash()], &codebase);

    fn check_parent(
        parent: NodePath,
        children: impl IntoIterator<Item = NodeHash>,
        codebase: &Codebase,
    ) {
        assert_eq!(
            codebase.node_at(&parent).node.kind(),
            &NodeKind::Error {
                node: "unresolved".to_string(),
                children: Children::new(children),
            },
        );
        assert_eq!(
            codebase.errors().get(&parent),
            Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
        );
    }
}

fn expect_error_on_multiple_children(token: &str, packages: &Packages) {
    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    compiler.replace(&compiler.codebase().root().path, token, packages);

    let a =
        compiler.insert_child(compiler.codebase().root().path, "", packages);
    let b =
        compiler.insert_child(compiler.codebase().root().path, "", packages);

    assert_eq!(
        compiler.codebase().root().node.kind(),
        &NodeKind::Error {
            node: token.to_string(),
            children: Children::new([a, b].map(|path| *path.hash())),
        },
    );

    let error = compiler.codebase().root().path;
    assert_eq!(
        compiler.codebase().errors().get(&error),
        Some(&CodeError::OnlyUpToOneChildAllowedForThisNode),
    );
}
