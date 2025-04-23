use crate::language::{
    code::Codebase,
    compiler::Compiler,
    editor::{Editor, EditorInputEvent},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::node,
};

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
