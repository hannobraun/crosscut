use crate::language::{
    code::{Codebase, NodeKind},
    compiler::Compiler,
    packages::Packages,
    runtime::Evaluator,
};

use super::{Editor, EditorInputEvent};

#[test]
fn edit_initial_node() {
    // The editor is initialized with a specific node it is currently editing.
    // That initialization should be correct, so editing that node actually
    // works.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(root, "12", &packages);
    }

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);
    editor.on_code("7", &mut codebase, &mut evaluator, &packages);

    assert_eq!(
        codebase.node_at(*editor.editing()).node.kind(),
        &NodeKind::integer_literal(127),
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

    let [a, b] = {
        let mut compiler = Compiler::new(&mut codebase);

        let a = compiler.insert_child(
            compiler.codebase().root().path,
            "a",
            &packages,
        );
        let b = compiler.insert_child(
            compiler.codebase().root().path,
            "b",
            &packages,
        );

        [a, b]
    };

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

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);

    editor.on_code("12\n7", &mut codebase, &mut evaluator, &packages);
    editor.on_input(
        EditorInputEvent::MoveCursorLeft,
        &mut codebase,
        &mut evaluator,
        &packages,
    );

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
            .map(|located_node| located_node.node.kind())
            .collect::<Vec<_>>(),
        vec![&NodeKind::integer_literal(127)],
    );
}

#[test]
fn merge_with_next_sibling() {
    // When removing right, while the cursor is at the end of a node, that
    // node should get merged with its next sibling.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);

    editor.on_code("12\n7", &mut codebase, &mut evaluator, &packages);
    for _ in 1..=2 {
        editor.on_input(
            EditorInputEvent::MoveCursorLeft,
            &mut codebase,
            &mut evaluator,
            &packages,
        );
    }

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
            .map(|located_node| located_node.node.kind())
            .collect::<Vec<_>>(),
        vec![&NodeKind::integer_literal(127)],
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
        Compiler::new(&mut codebase).replace(root, "127255", &packages);
    }

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);

    for _ in "255".chars() {
        editor.on_input(
            EditorInputEvent::MoveCursorLeft,
            &mut codebase,
            &mut evaluator,
            &packages,
        );
    }
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
            .map(|located_node| located_node.node.kind())
            .collect::<Vec<_>>(),
        vec![
            &NodeKind::integer_literal(127),
            &NodeKind::integer_literal(255),
        ],
    );
}
