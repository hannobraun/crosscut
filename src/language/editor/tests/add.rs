use crate::language::{
    code::Codebase,
    compiler::Compiler,
    editor::{Editor, EditorInputEvent::*},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{LocatedNodeExt, NodeExt, node},
};

#[test]
fn add_child() {
    // It's possible to add a child to the current node.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let a = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "a", &packages)
    };

    let mut editor = Editor::new(a, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddChildOrParent],
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let [b] = codebase.root().expect_children(codebase.nodes());
    assert_eq!(codebase.root().node, &node("a", [*b.path.hash()]));
    assert_eq!(b.node, &node("b", []));
}

#[test]
fn split_node_to_create_child() {
    // If we add a child while the cursor is in the middle of the current node,
    // we should split the node right there.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let root = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "ab", &packages)
    };

    let mut editor = Editor::new(root, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddChildOrParent],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    let a = codebase.root();
    let [b] = a.expect_children(codebase.nodes());
    assert_eq!(a.node, &node("a", [*b.path.hash()]));
    assert_eq!(b.node, &node("b", []));
}

#[test]
fn add_parent_of_node_that_already_has_a_parent() {
    // If a node already has a parent, then adding a parent should add the
    // parent in between them, as a child of the previous parent.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let c = {
        let mut compiler = Compiler::new(&mut codebase);

        let a =
            compiler.replace(&compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(a, "c", &packages)
    };

    let mut editor = Editor::postfix(c, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddChildOrParent],
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let [b] = codebase.root().expect_children(codebase.nodes());
    let [c] = b.expect_children(codebase.nodes());
    assert_eq!(codebase.root().node, &node("a", [*b.path.hash()]));
    assert_eq!(b.node, &node("b", [*c.path.hash()]));
    assert_eq!(c.node, &node("c", []));
}

#[test]
fn split_node_to_create_sibling() {
    // When adding a sibling while the cursor is in the middle of a node, the
    // node should be split.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "ab", &packages);
    }

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddSibling],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(
        codebase
            .root()
            .children(codebase.nodes())
            .map(|located_node| located_node.node)
            .collect::<Vec<_>>(),
        vec![&node("a", []), &node("b", [])],
    );
}

#[test]
fn reuse_empty_error_node_for_parent() {
    // If the parent of a node is empty, that node is re-used when adding a
    // parent, instead of creating a new parent node. The same should be true
    // for error nodes that happen to be empty.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    // The initial root node should be empty. Let's make sure nothing went wrong
    // with the test setup, and this is actually the case.
    codebase.root().node.expect_error("");

    let [a, b] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor = Editor::postfix(b.clone(), &codebase, &packages);
    editor.on_input(
        [MoveCursorRight],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    // Now tell the editor to create a parent node.
    editor.on_code(" ", &mut codebase, &mut evaluator, &packages);

    // And check that it has actually re-used the root node.
    assert_eq!(a.parent(), Some(&codebase.root().path));
    assert_eq!(b.parent(), Some(&codebase.root().path));
}
