use itertools::Itertools;

use crate::language::{
    code::Codebase,
    compiler::Compiler,
    editor::{Editor, EditorInputEvent},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{NodeExt, node},
};

#[test]
fn add_parent_node() {
    // It's possible to add a child to the current node.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let child = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "a", &packages)
    };

    let mut editor = Editor::new(child, &codebase, &packages);

    editor.on_input(
        EditorInputEvent::MoveCursorRight,
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_input(
        EditorInputEvent::AddChildOrParent,
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let b = codebase.root().children(codebase.nodes()).next().unwrap();
    assert_eq!(codebase.root().node, &node("a", [*b.path.hash()]));
    assert_eq!(b.node, &node("b", []));
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
        EditorInputEvent::MoveCursorRight,
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_input(
        EditorInputEvent::AddSibling,
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
        .children(codebase.nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();

    let mut editor = Editor::postfix(b.clone(), &codebase, &packages);
    editor.on_input(
        EditorInputEvent::MoveCursorRight,
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
