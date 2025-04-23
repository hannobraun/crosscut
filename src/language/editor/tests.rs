use itertools::Itertools;

use crate::language::{
    code::{Codebase, Node},
    compiler::Compiler,
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{NodeExt, node},
};

use super::{Editor, EditorInputEvent};

#[test]
fn edit_initial_node() {
    // The editor is initialized with the specific node it is currently editing.
    // That initialization should be correct, so editing that node actually
    // works.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "27", &packages);
    }

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);
    assert_eq!(editor.editing(), &codebase.root().path);

    editor.on_code("1", &mut codebase, &mut evaluator, &packages);
    assert_eq!(
        codebase.node_at(editor.editing()).node,
        &Node::LiteralNumber { value: 127 },
    );
}

// There are some test cases missing right around here, about navigating to the
// previous node, and probably more detail on navigating to the next node.
//
// Those test cases exist, as part of the higher-level `editing` suite. They are
// probably misplaced there. See comment in that module.
//
// In case those test cases need to be changed in a significant way, it probably
// makes more sense to port them here first. But as long as they stay the same,
// that might not be worth the effort.

#[test]
fn navigate_to_next_sibling() {
    // Moving the cursor down should navigate to the next sibling.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    let [a, b] = codebase
        .root()
        .children(codebase.nodes())
        .map(|located_node| located_node.path)
        .collect_array()
        .unwrap();

    let mut editor = Editor::new(a, &codebase, &packages);
    editor.on_input(
        EditorInputEvent::MoveCursorDown,
        &mut codebase,
        &mut evaluator,
        &Packages::new(),
    );

    assert_eq!(editor.editing(), &b);
}

#[test]
fn merge_with_previous_sibling() {
    // When removing left, while the cursor is at the beginning of a node, that
    // node should get merged with its previous sibling.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let b = {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages)
    };

    let mut editor = Editor::new(b, &codebase, &packages);

    editor.on_input(
        EditorInputEvent::RemoveLeft { whole_node: false },
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
        vec![&node("ab", [])],
    );
}

#[test]
fn merge_with_next_sibling() {
    // When removing right, while the cursor is at the end of a node, that
    // node should get merged with its next sibling.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    let a = codebase.root().children(codebase.nodes()).next().unwrap();
    let mut editor = Editor::new(a.path, &codebase, &packages);

    editor.on_input(
        EditorInputEvent::MoveCursorRight,
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_input(
        EditorInputEvent::RemoveRight { whole_node: false },
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
        vec![&node("ab", [])],
    );
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
