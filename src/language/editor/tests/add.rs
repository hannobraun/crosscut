use crate::language::{
    code::{Codebase, Expression, Function},
    compiler::Compiler,
    editor::{Editor, EditorInputEvent::*, editor::Cursor},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{LocatedNodeExt, error, tuple},
};

#[test]
fn add_apply_node() {
    // Adding an `apply` node also creates its children.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);
    editor.on_code("apply", &mut codebase, &mut evaluator, &packages);

    let apply = codebase.root();
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert_eq!(
        apply.node,
        &Expression::Apply {
            expression: *function.path.hash(),
            argument: *argument.path.hash(),
        },
    );
    assert_eq!(function.node, &Expression::Empty);
    assert_eq!(argument.node, &Expression::Empty);

    // The apply node's children can then be edited.

    editor.on_input([MoveCursorDown], &mut codebase, &mut evaluator, &packages);
    editor.on_code("a", &mut codebase, &mut evaluator, &packages);

    editor.on_input([MoveCursorDown], &mut codebase, &mut evaluator, &packages);
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let apply = codebase.root();
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert_eq!(
        apply.node,
        &Expression::Apply {
            expression: *function.path.hash(),
            argument: *argument.path.hash(),
        },
    );
    assert_eq!(function.node, &error("a", []));
    assert_eq!(argument.node, &error("b", []));
}

#[test]
fn add_fn_node() {
    // Adding an `fn` node also creates its children.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase, &packages);
    editor.on_code("fn", &mut codebase, &mut evaluator, &packages);

    let function = codebase.root();
    let [parameter, body] = function.expect_children(codebase.nodes());

    assert_eq!(
        function.node,
        &Expression::Function {
            function: Function {
                parameter: *parameter.path.hash(),
                body: *body.path.hash(),
            },
        },
    );
    assert_eq!(parameter.node, &Expression::Empty);
    assert_eq!(body.node, &Expression::Empty);
}

#[test]
fn add_child() {
    // It's possible to add a child to the current node.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let parent = {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "tuple", &packages)
    };

    let mut editor = Editor::new(
        Cursor {
            path: parent,
            index: "tuple".len(),
        },
        &codebase,
        &packages,
    );

    editor.on_input([AddChild], &mut codebase, &mut evaluator, &packages);
    editor.on_code("child", &mut codebase, &mut evaluator, &packages);

    let parent = codebase.root();
    let [child] = parent.expect_children(codebase.nodes());

    assert_eq!(parent.node, &tuple([*child.path.hash()]));
    assert_eq!(child.node, &error("child", []));
}

#[test]
fn add_sibling() {
    // It is possible to add a sibling to a node.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let a = {
        let mut compiler = Compiler::new(&mut codebase);

        let parent = compiler.replace(
            &compiler.codebase().root().path,
            "tuple",
            &packages,
        );
        compiler.insert_child(parent, "a", &packages)
    };

    let mut editor = Editor::new(a, &codebase, &packages);

    editor.on_input(
        [MoveCursorRight, AddSibling],
        &mut codebase,
        &mut evaluator,
        &packages,
    );
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let parent = codebase.root();
    let [a, b] = parent.expect_children(codebase.nodes());

    assert_eq!(parent.node, &tuple([*a.path.hash(), *b.path.hash()]));
    assert_eq!(a.node, &error("a", []));
    assert_eq!(b.node, &error("b", []));
}

#[test]
fn split_node_to_create_sibling() {
    // When adding a sibling while the cursor is in the middle of a node, the
    // node should be split.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.replace(&compiler.codebase().root().path, "root", &packages);
        compiler.insert_child(compiler.codebase().root().path, "ab", &packages);
    }

    let [ab] = codebase.root().expect_children(codebase.nodes());
    let mut editor = Editor::new(
        Cursor {
            path: ab.path,
            index: 1,
        },
        &codebase,
        &packages,
    );

    editor.on_input([AddSibling], &mut codebase, &mut evaluator, &packages);

    assert_eq!(
        codebase
            .root()
            .children(codebase.nodes())
            .map(|located_node| located_node.node)
            .collect::<Vec<_>>(),
        vec![&error("a", []), &error("b", [])],
    );
}
