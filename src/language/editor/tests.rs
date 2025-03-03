use crate::language::{
    code::{Codebase, Node, NodeKind},
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
        Compiler::new(&mut codebase).replace(&root, "12", &packages);
    }

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);
    editor.on_code("7", &mut codebase, &mut evaluator, &packages);

    assert_eq!(
        codebase.node_at(editor.editing()).kind(),
        &NodeKind::integer_literal(127),
    );
}

#[test]
#[should_panic] // missing feature that is being worked on
fn navigate_to_next_sibling() {
    // Moving the cursor down should navigate to the next sibling.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let [a, b] = ["a", "b"].map(|node| {
        codebase.insert_node_as_child(&codebase.root().path, Node::error(node))
    });

    let mut editor = Editor::new(a, &codebase, &packages);
    editor.on_input(
        EditorInputEvent::MoveCursorDown,
        &mut codebase,
        &mut evaluator,
        &Packages::new(),
    );

    assert_eq!(editor.editing(), &b);
}
