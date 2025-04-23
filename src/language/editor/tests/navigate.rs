use crate::language::{
    code::{Codebase, Node},
    compiler::Compiler,
    editor::{Editor, EditorInputEvent},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::LocatedNodeExt,
};

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
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor = Editor::new(a, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorDown],
        &mut codebase,
        &mut evaluator,
        &Packages::new(),
    );

    assert_eq!(editor.editing(), &b);
}
