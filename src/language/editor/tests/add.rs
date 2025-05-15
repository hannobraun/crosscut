use crate::language::{
    code::{Codebase, SyntaxNode},
    editor::{Editor, EditorInputEvent::*},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{ExpectChildren, unresolved},
};

#[test]
fn add_apply_node() {
    // Adding an `apply` node also creates its children.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("apply", &mut codebase, &mut evaluator, &packages);

    let apply = codebase.root();
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert_eq!(
        apply.node,
        &SyntaxNode::Apply {
            expression: *function.path.hash(),
            argument: *argument.path.hash(),
        },
    );
    assert_eq!(function.node, &SyntaxNode::Empty);
    assert_eq!(argument.node, &SyntaxNode::Empty);

    // The apply node's children can then be edited.

    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator, &packages);
    editor.on_code("a", &mut codebase, &mut evaluator, &packages);

    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator, &packages);
    editor.on_code("b", &mut codebase, &mut evaluator, &packages);

    let apply = codebase.root();
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert_eq!(
        apply.node,
        &SyntaxNode::Apply {
            expression: *function.path.hash(),
            argument: *argument.path.hash(),
        },
    );
    assert_eq!(function.node, &unresolved("a"));
    assert_eq!(argument.node, &unresolved("b"));
}

#[test]
fn add_fn_node() {
    // Adding an `fn` node also creates its children.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("fn", &mut codebase, &mut evaluator, &packages);

    let function = codebase.root();
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
            name: "_".to_string()
        }
    );
    assert_eq!(body.node, &SyntaxNode::Empty);
}

#[test]
fn add_value_to_tuple() {
    // Tuples have a node that the user can edit to add children.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let mut editor = Editor::new(codebase.root().path, &codebase);
    editor.on_code("tuple", &mut codebase, &mut evaluator, &packages);

    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator, &packages);
    editor.on_code("1", &mut codebase, &mut evaluator, &packages);

    let tuple = codebase.root();
    let [value, _] = tuple.expect_children(codebase.nodes());

    assert!(matches!(tuple.node, &SyntaxNode::Tuple { .. }));
    assert_eq!(value.node, &SyntaxNode::Number { value: 1 });
}
