use crate::language::{
    code::{Children, CodeError, Codebase, Expression, NodeHash, NodePath},
    compiler::Compiler,
    packages::{Function, Packages},
    tests::infra::{LocatedNodeExt, node},
};

#[test]
fn insert_child() {
    // The compiler can insert a child node.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
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
    let mut compiler = Compiler::new(&mut codebase);

    let a =
        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
    let b = compiler.insert_child(a.clone(), "b", &packages);

    let [child_of_root] = compiler
        .codebase()
        .root()
        .expect_children(compiler.codebase().nodes());
    let [grandchild_of_root] =
        child_of_root.expect_children(compiler.codebase().nodes());
    assert_eq!(grandchild_of_root.path, b);
}

#[test]
fn insert_child_should_update_errors() {
    // Inserting an erroneous child should insert an error into the codebase.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    let unresolved = compiler.insert_child(
        compiler.codebase().root().path,
        "unresolved",
        &packages,
    );

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

    let root = codebase.root().path;
    codebase.make_change(|change_set| {
        let child = change_set.nodes_mut().insert(node("child", []));

        let parent = change_set
            .nodes_mut()
            .insert(node("parent", [child, child]));

        change_set.replace(&root, &NodePath::for_root(parent));
    });

    let [_, child] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut compiler = Compiler::new(&mut codebase);
    compiler.replace(&child, "updated", &packages);

    let [child, updated] = codebase.root().expect_children(codebase.nodes());

    assert_eq!(child.node, &node("child", []));
    assert_eq!(updated.node, &node("updated", []));
}

#[test]
fn empty_node_with_multiple_children_is_an_error() {
    // An empty node has rather obvious runtime semantics: Do nothing and just
    // pass on the active value unchanged.
    //
    // If an empty node has multiple children, then it's no longer obvious what
    // it should do. So that needs to be an error.

    let packages = Packages::default();

    expect_error_on_multiple_children("", &packages);
}

#[test]
fn provided_function_application_with_multiple_children_is_an_error() {
    // A provided function application can only have one child: its argument.

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Provided;
    impl Function for Provided {
        fn name(&self) -> &str {
            "provided"
        }
    }

    let mut packages = Packages::default();
    packages.new_package([Provided]);

    expect_error_on_multiple_children("provided", &packages);
}

#[test]
fn self_keyword_with_multiple_children_is_an_error() {
    // A self keyword can only have one child: the argument for the function
    // that it calls recursively.

    let packages = Packages::default();

    expect_error_on_multiple_children("self", &packages);
}

#[test]
fn integer_literal_with_children_is_an_error() {
    // An integer literal already carries all of the information that it needs
    // to evaluate to an integer. There is nothing it could do with children,
    // except ignore them.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    compiler.replace(&compiler.codebase().root().path, "127", &packages);
    compiler.insert_child(compiler.codebase().root().path, "", &packages);

    let root = compiler.codebase().root();
    if let Expression::Error { node, .. } = root.node {
        assert_eq!(node, "127");
    } else {
        panic!();
    }
    assert_eq!(
        compiler.codebase().errors().get(root.path.hash()),
        Some(&CodeError::TooManyChildren),
    );
}

#[test]
fn updating_child_updates_parent() {
    // If the child of a parent node is being updated, the parent node should be
    // updated as well.

    let packages = Packages::default();

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
            &Expression::Error {
                node: "unresolved".to_string(),
                children: Children::new(children),
            },
        );
        assert_eq!(
            codebase.errors().get(parent.hash()),
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
        compiler.codebase().root().node,
        &Expression::Error {
            node: token.to_string(),
            children: Children::new([a, b].map(|path| *path.hash())),
        },
    );

    let error = compiler.codebase().root().path;
    assert_eq!(
        compiler.codebase().errors().get(error.hash()),
        Some(&CodeError::TooManyChildren),
    );
}
