use crate::language::{
    code::{Codebase, SyntaxNode},
    editor::{Editor, EditorInputEvent::*},
    runtime::Evaluator,
    tests::infra::{ExpectChildren, identifier},
};

#[test]
fn add_nodes_to_root_context() {
    // It is possible to add multiple nodes to the root context.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("a", &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("b", &mut codebase, &mut evaluator);

    let [a, b, _] = codebase.root().expect_children(codebase.nodes());

    assert_eq!(a.node, &identifier("a"));
    assert_eq!(b.node, &identifier("b"));
}

#[test]
fn add_apply_node() {
    // Adding an `apply` node also creates its children.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("apply", &mut codebase, &mut evaluator);

    let [apply, _] = codebase.root().expect_children(codebase.nodes());
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert!(matches!(apply.node, &SyntaxNode::Apply { .. }));
    assert_eq!(function.node, &SyntaxNode::Empty);
    assert_eq!(argument.node, &SyntaxNode::Empty);

    // The apply node's children can then be edited.

    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("a", &mut codebase, &mut evaluator);

    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("b", &mut codebase, &mut evaluator);

    let [apply, _] = codebase.root().expect_children(codebase.nodes());
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert_eq!(function.node, &identifier("a"));
    assert_eq!(argument.node, &identifier("b"));
}

#[test]
fn add_function() {
    // Adding an `fn` node also creates its children.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("fn", &mut codebase, &mut evaluator);

    let [function, _] = codebase.root().expect_children(codebase.nodes());
    let [parameter, body] = function.expect_children(codebase.nodes());

    assert_eq!(
        function.node,
        &SyntaxNode::Function {
            parameter: *parameter.path.hash(),
            body: *body.path.hash(),
        },
    );
    assert_eq!(
        parameter.node,
        &SyntaxNode::Binding {
            name: "".to_string()
        }
    );
    assert!(matches!(body.node, &SyntaxNode::Body { .. }));
}

#[test]
fn add_value_to_tuple() {
    // Tuples have a node that the user can edit to add children.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("tuple", &mut codebase, &mut evaluator);

    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("1", &mut codebase, &mut evaluator);

    let [tuple, _] = codebase.root().expect_children(codebase.nodes());
    let [body] = tuple.expect_children(codebase.nodes());
    let [value, _] = body.expect_children(codebase.nodes());

    assert!(matches!(tuple.node, &SyntaxNode::Tuple { .. }));
    assert_eq!(value.node, &SyntaxNode::Number { value: 1 });
}
