use crate::language::{
    code::Codebase,
    compiler::Compiler,
    editor::{Editor, EditorInputEvent, editor::Cursor},
    packages::Packages,
    runtime::Evaluator,
    tests::infra::{LocatedNodeExt, error},
};

#[test]
fn edit_at_initial_cursor() {
    // The editor is initialized with the specific node it is currently editing.
    // That initialization should be correct, so editing that node actually
    // works.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let root = codebase.root().path;
        Compiler::new(&mut codebase).replace(&root, "ac", &packages);
    }

    let cursor = Cursor {
        path: codebase.root().path,
        index: 1,
    };
    let mut editor = Editor::new(cursor.clone(), &codebase, &packages);
    assert_eq!(editor.cursor(), &cursor);

    editor.on_code("b", &mut codebase, &mut evaluator, &packages);
    assert_eq!(codebase.node_at(&editor.cursor().path).node, &error("abc"));
}

#[test]
fn navigate_down_to_child() {
    // Moving the cursor down navigates to the current node's child.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let child = {
        let mut compiler = Compiler::new(&mut codebase);

        let parent = compiler.replace(
            &compiler.codebase().root().path,
            "parent",
            &packages,
        );
        compiler.insert_child(parent, "child", &packages)
    };

    let mut editor = Editor::new(
        Cursor {
            path: codebase.root().path,
            index: 0,
        },
        &codebase,
        &packages,
    );
    editor.on_input(
        [EditorInputEvent::MoveCursorDown],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: child,
            index: 0,
        },
    );
}

#[test]
fn navigate_right_to_child() {
    // Moving the cursor right while at the end of the current node navigates to
    // the child.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let b = {
        let mut compiler = Compiler::new(&mut codebase);

        let a =
            compiler.replace(&compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(a, "b", &packages)
    };

    let mut editor = Editor::new(
        Cursor {
            path: codebase.root().path,
            index: 1,
        },
        &codebase,
        &packages,
    );
    editor.on_input(
        [EditorInputEvent::MoveCursorRight],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(editor.cursor(), &Cursor { path: b, index: 0 });
}

#[test]
fn navigate_up_to_parent() {
    // Moving the cursor up navigates to the current node's parent.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let b = {
        let mut compiler = Compiler::new(&mut codebase);

        let a =
            compiler.replace(&compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(a, "b", &packages)
    };

    let mut editor =
        Editor::new(Cursor { path: b, index: 1 }, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorUp],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: codebase.root().path,
            index: 1
        },
    );
}

#[test]
fn navigate_left_to_parent() {
    // Moving the cursor left navigates to the end of the current node's parent.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    let b = {
        let mut compiler = Compiler::new(&mut codebase);

        let a =
            compiler.replace(&compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(a, "b", &packages)
    };

    let mut editor =
        Editor::new(Cursor { path: b, index: 0 }, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorLeft],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(
        editor.cursor(),
        &Cursor {
            path: codebase.root().path,
            index: 1
        },
    );
}

#[test]
fn navigate_down_to_next_sibling() {
    // Moving the cursor down navigates to the current node's next sibling.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.replace(&compiler.codebase().root().path, "root", &packages);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    let [a, b] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor =
        Editor::new(Cursor { path: a, index: 0 }, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorDown],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(editor.cursor(), &Cursor { path: b, index: 0 });
}

#[test]
fn navigate_right_to_next_sibling() {
    // Moving the cursor right while at the end of a node that has no child,
    // navigates to the next sibling.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.replace(&compiler.codebase().root().path, "root", &packages);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    let [a, b] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor =
        Editor::new(Cursor { path: a, index: 1 }, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorRight],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(editor.cursor(), &Cursor { path: b, index: 0 });
}

#[test]
fn navigate_up_to_previous_sibling() {
    // Moving the cursor up navigates to the current node's previous sibling.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.replace(&compiler.codebase().root().path, "root", &packages);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    let [a, b] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor =
        Editor::new(Cursor { path: b, index: 1 }, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorUp],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(editor.cursor(), &Cursor { path: a, index: 1 });
}

#[test]
fn navigate_left_to_previous_sibling() {
    // Moving the cursor left navigates to the end of the current node's
    // previous sibling.

    let packages = Packages::default();

    let mut codebase = Codebase::new();
    let mut evaluator = Evaluator::new();

    {
        let mut compiler = Compiler::new(&mut codebase);

        compiler.replace(&compiler.codebase().root().path, "root", &packages);

        compiler.insert_child(compiler.codebase().root().path, "a", &packages);
        compiler.insert_child(compiler.codebase().root().path, "b", &packages);
    }

    let [a, b] = codebase
        .root()
        .expect_children(codebase.nodes())
        .map(|located_node| located_node.path);

    let mut editor =
        Editor::new(Cursor { path: b, index: 0 }, &codebase, &packages);
    editor.on_input(
        [EditorInputEvent::MoveCursorLeft],
        &mut codebase,
        &mut evaluator,
        &packages,
    );

    assert_eq!(editor.cursor(), &Cursor { path: a, index: 1 });
}
