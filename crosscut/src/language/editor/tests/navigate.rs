use crate::language::{
    code::{Codebase, SyntaxNode},
    compiler::Compiler,
    editor::{Editor, EditorInput::*, editor::Cursor},
    runtime::Evaluator,
    tests::infra::{ExpectChildren, identifier},
};

#[test]
fn edit_at_initial_cursor() {
    // The editor is initialized with the specific node it is currently editing.
    // That initialization should be correct, so editing that node actually
    // works.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "ac");
    }

    let cursor = Cursor {
        path: codebase.root().path,
        index: 1,
    };
    let mut editor = Editor::new(cursor.clone(), &codebase);
    assert_eq!(editor.cursor(), &cursor);

    editor.on_code("b", &mut codebase, &mut evaluator);
    assert_eq!(
        codebase.node_at(&editor.cursor().path).node,
        &identifier("abc"),
    );
}

#[test]
fn navigate_down_to_child() {
    // Moving the cursor down navigates to the current node's child.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);
        compiler.replace(&compiler.codebase().root().path, "fn");
    }

    let mut editor = Editor::new(
        Cursor {
            path: codebase.root().path,
            index: 0,
        },
        &codebase,
    );
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);

    let [child, _] = codebase.root().expect_children(codebase.nodes());
    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: child.path,
            index: 0,
        },
    );
}

#[test]
fn navigate_right_to_child() {
    // Moving the cursor right while at the end of the current node navigates to
    // the child.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);
        compiler.replace(&compiler.codebase().root().path, "fn");
    }

    let mut editor = Editor::new(
        Cursor {
            path: codebase.root().path,
            index: "fn".len(),
        },
        &codebase,
    );
    editor.on_input(MoveCursorRight, &mut codebase, &mut evaluator);

    let [child, _] = codebase.root().expect_children(codebase.nodes());
    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: child.path,
            index: 0,
        },
    );
}

#[test]
fn navigate_up_to_parent() {
    // Moving the cursor up navigates to the current node's parent.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);
        compiler.replace(&compiler.codebase().root().path, "fn");
    }

    let [child, _] = codebase.root().expect_children(codebase.nodes());
    let mut editor = Editor::new(
        Cursor {
            path: child.path,
            index: 1,
        },
        &codebase,
    );
    editor.on_input(MoveCursorUp, &mut codebase, &mut evaluator);

    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: codebase.root().path,
            index: "fn".len(),
        },
    );
}

#[test]
fn navigate_left_to_parent() {
    // Moving the cursor left navigates to the end of the current node's parent.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);
        compiler.replace(&compiler.codebase().root().path, "fn");
    }

    let [child, _] = codebase.root().expect_children(codebase.nodes());
    let mut editor = Editor::new(
        Cursor {
            path: child.path,
            index: 0,
        },
        &codebase,
    );
    editor.on_input(MoveCursorLeft, &mut codebase, &mut evaluator);

    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: codebase.root().path,
            index: "fn".len(),
        },
    );
}

#[test]
fn navigate_down_to_next_sibling() {
    // Moving the cursor down navigates to the current node's next sibling.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a");
        compiler.insert_child(compiler.codebase().root().path, "b");
    }

    let [a, b, _] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor = Editor::new(Cursor { path: a, index: 0 }, &codebase);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);

    assert_eq!(editor.cursor(), &Cursor { path: b, index: 0 });
}

#[test]
fn navigate_right_to_next_sibling() {
    // Moving the cursor right while at the end of a node that has no child,
    // navigates to the next sibling.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a");
        compiler.insert_child(compiler.codebase().root().path, "b");
    }

    let [a, b, _] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor = Editor::new(Cursor { path: a, index: 1 }, &codebase);
    editor.on_input(MoveCursorRight, &mut codebase, &mut evaluator);

    assert_eq!(editor.cursor(), &Cursor { path: b, index: 0 });
}

#[test]
fn navigate_up_to_previous_sibling() {
    // Moving the cursor up navigates to the current node's previous sibling.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a");
        compiler.insert_child(compiler.codebase().root().path, "b");
    }

    let [a, b, _] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor = Editor::new(Cursor { path: b, index: 1 }, &codebase);
    editor.on_input(MoveCursorUp, &mut codebase, &mut evaluator);

    assert_eq!(editor.cursor(), &Cursor { path: a, index: 1 });
}

#[test]
fn navigate_left_to_previous_sibling() {
    // Moving the cursor left navigates to the end of the current node's
    // previous sibling.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.insert_child(compiler.codebase().root().path, "a");
        compiler.insert_child(compiler.codebase().root().path, "b");
    }

    let [a, b, _] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor = Editor::new(Cursor { path: b, index: 0 }, &codebase);
    editor.on_input(MoveCursorLeft, &mut codebase, &mut evaluator);

    assert_eq!(editor.cursor(), &Cursor { path: a, index: 1 });
}

#[test]
fn navigate_past_add_value_node_of_a_tuple() {
    // Navigating past the "add value" node of a tuple should not add any
    // values.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    let mut editor = Editor::new(
        Cursor {
            path: codebase.root().path,
            index: 0,
        },
        &codebase,
    );

    editor.on_code("apply", &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("tuple", &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("arg", &mut codebase, &mut evaluator);

    let [apply, _] = codebase.root().expect_children(codebase.nodes());
    let [_tuple, arg] = apply
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: arg,
            index: "arg".len()
        }
    );
}

#[test]
fn navigating_to_node_should_not_reset_children() {
    // When navigating to a node that already has children, and not editing it,
    // the children should stay as they are.

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::default();

    let mut editor = Editor::new(
        Cursor {
            path: codebase.root().path,
            index: 0,
        },
        &codebase,
    );

    editor.on_code("apply", &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("function", &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorDown, &mut codebase, &mut evaluator);
    editor.on_code("argument", &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorUp, &mut codebase, &mut evaluator);
    editor.on_input(MoveCursorUp, &mut codebase, &mut evaluator);

    let [apply, _] = codebase.root().expect_children(codebase.nodes());
    let [function, argument] = apply.expect_children(codebase.nodes());

    assert!(matches!(apply.node, SyntaxNode::Apply { .. }));
    assert!(matches!(function.node, SyntaxNode::Identifier { .. }));
    assert!(matches!(argument.node, SyntaxNode::Identifier { .. }));
}
