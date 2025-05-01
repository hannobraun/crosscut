use crate::language::{
    code::{CodeError, Codebase, Expression, NodeHash, NodePath},
    compiler::Compiler,
    packages::Packages,
    tests::infra::{LocatedNodeExt, error, expression},
};

#[test]
fn insert_child() {
    // The compiler can insert a child node.

    let packages = Packages::default();
    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent = change_set.nodes.insert(expression("parent", []));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let mut compiler = Compiler::new(&mut codebase);

    let a =
        compiler.insert_child(compiler.codebase().root().path, "a", &packages);

    let [child_of_root] = compiler
        .codebase()
        .root()
        .expect_children(compiler.codebase().nodes());
    assert_eq!(child_of_root.path, a);
}

#[test]
fn insert_child_with_grandparent() {
    // Inserting a child still works, if that child's parent has a parent
    // itself.

    let packages = Packages::default();
    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let parent = change_set.nodes.insert(expression("parent", []));
        let grandparent =
            change_set.nodes.insert(expression("grandparent", [parent]));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(grandparent),
        );
    });

    let mut compiler = Compiler::new(&mut codebase);

    let grandparent = compiler.codebase().root();
    let [parent] = grandparent.expect_children(compiler.codebase().nodes());
    let child = compiler.insert_child(parent.path, "child", &packages);

    let [child_of_root] = compiler
        .codebase()
        .root()
        .expect_children(compiler.codebase().nodes());
    let [grandchild_of_root] =
        child_of_root.expect_children(compiler.codebase().nodes());
    assert_eq!(grandchild_of_root.path, child);
}

#[test]
fn insert_child_should_update_errors() {
    // Inserting an erroneous child should insert an error into the codebase.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let root =
        compiler.replace(&compiler.codebase().root().path, "root", &packages);
    let unresolved = compiler.insert_child(root, "unresolved", &packages);

    assert_eq!(
        compiler.codebase().errors().get(unresolved.hash()),
        Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
    );
}

#[test]
fn replace_second_of_two_equal_children() {
    // If two children are equal, and one is replaced, the replacement logic
    // should correctly distinguish between them.

    let packages = Packages::default();

    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let child = change_set.nodes.insert(error("child"));

        let parent = change_set
            .nodes
            .insert(expression("parent", [child, child]));

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let [_, child] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut compiler = Compiler::new(&mut codebase);
    compiler.replace(&child, "updated", &packages);

    let [child, updated] = codebase.root().expect_children(codebase.nodes());

    assert_eq!(child.node, &error("child"));
    assert_eq!(updated.node, &error("updated"));
}

#[test]
fn updating_child_updates_parent() {
    // If the child of a parent node is being updated, the parent node should be
    // updated as well.

    let packages = Packages::default();
    let mut codebase = Codebase::new();

    codebase.make_change(|change_set| {
        let child = change_set.nodes.insert(Expression::Number { value: 12 });
        let parent = change_set.nodes.insert(expression("unresolved", [child]));

        change_set.errors.insert(
            parent,
            CodeError::UnresolvedIdentifier {
                candidates: Vec::new(),
            },
        );

        change_set.replace(
            &change_set.root_before_change(),
            &NodePath::for_root(parent),
        );
    });

    let mut compiler = Compiler::new(&mut codebase);

    let child = compiler
        .codebase()
        .root()
        .children(compiler.codebase().nodes())
        .next()
        .unwrap();
    let child = compiler.replace(&child.path, "127", &packages);

    // After editing the child, the new parent node should be the same as the
    // old one, but with an updated child.
    let parent = compiler.codebase().root().path;
    check_parent(parent, [*child.hash()], &codebase);

    fn check_parent(
        parent: NodePath,
        children: impl IntoIterator<Item = NodeHash<Expression>>,
        codebase: &Codebase,
    ) {
        assert_eq!(
            codebase.node_at(&parent).node,
            &expression("unresolved", children)
        );

        // Since a change to a child doesn't change anything substantial about
        // the parent, any errors that had previously need to be preserved.
        assert_eq!(
            codebase.errors().get(parent.hash()),
            Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
        );
    }
}
