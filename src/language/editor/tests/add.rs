use crate::language::{
    code::Codebase,
    compiler::Compiler,
    editor::{Editor, EditorInputEvent::*},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{LocatedNodeExt, node},
};

#[test]
fn add_child() {
    // It's possible to add a child to the current node.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let a = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "a", &packages)
    };

    let mut editor = Editor::new(a, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddChild],
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let a = codebase.root();
    let [b] = a.expect_children(codebase.nodes());

    assert_eq!(a.node, &node("a", [*b.path.hash()]));
    assert_eq!(b.node, &node("b", []));
}

#[test]
fn add_sibling() {
    // It is possible to add a sibling to a node.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let b = {
        let mut compiler = Compiler::new(&mut codebase);

        let a =
            compiler.replace(&compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(a, "b", &packages)
    };

    let mut editor = Editor::new(b, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddSibling],
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("c", &mut codebase, &mut evaluator, &packages);

    let a = codebase.root();
    let [b, c] = a.expect_children(codebase.nodes());

    assert_eq!(a.node, &node("a", [*b.path.hash(), *c.path.hash()]));
    assert_eq!(b.node, &node("b", []));
    assert_eq!(c.node, &node("c", []));
}

#[test]
fn add_sibling_to_root_node() {
    // If adding a sibling to the root node, there still needs to be a single
    // root node afterwards. So a new one is created automatically.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let a = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "a", &packages)
    };

    let mut editor = Editor::new(a, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddSibling],
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let [a, b] = codebase.root().expect_children(codebase.nodes());

    assert_eq!(
        codebase.root().node,
        &node("", [*a.path.hash(), *b.path.hash()]),
    );
    assert_eq!(a.node, &node("a", []));
    assert_eq!(b.node, &node("b", []));
}

#[test]
fn split_node_to_create_child() {
    // If we add a child while the cursor is in the middle of the current node,
    // we should split the node right there.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let root = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "ab", &packages)
    };

    let mut editor = Editor::new(root, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddChild],
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
fn split_node_to_create_sibling() {
    // When adding a sibling while the cursor is in the middle of a node, the
    // node should be split.

    let packages = Packages::default();

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
