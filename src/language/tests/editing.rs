use itertools::Itertools;

use crate::language::{
    code::Node,
    editor::EditorInputEvent,
    language::Language,
    runtime::Value,
    tests::infra::{NodeExt, NodesExt},
};

// Some tests in this suite have gotten a bit too detailed, as indicated by
// setup code which is sophisticated enough to need its own testing.
//
// There are other, lower-level test suites now, which are more suited to this
// kind of detailed test. If you're working on adding a new test here, which
// turns out too cumbersome, consider adding it somewhere else. If you're
// modifying a test that is already here, consider porting it to one of the
// other test suites first.

#[test]
fn update_on_every_character() {
    // The editor should compile the code on every new character. If the program
    // has finished running, as is the case here, it should also reset the
    // interpreter, so the next step will run the new code.

    let mut language = Language::new();

    language.on_code("1");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 1 },
    );

    language.on_code("2");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 12 },
    );
}

#[test]
fn update_after_removing_character() {
    // Removing a character should have an immediate effect on the program, just
    // like adding one.

    let mut language = Language::new();

    language.on_code("127");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 12 },
    );

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 1 },
    );
}

#[test]
fn update_after_removing_all_characters() {
    // Removing all characters should have an immediate effect on the program,
    // just like any other edits.

    let mut language = Language::new();

    language.on_code("1");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 1 },
    );

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });
    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());
}

#[test]
fn moving_cursor_up_should_navigate_to_child_node() {
    // It is possible to navigate to the previous node in the editor.

    let mut language = Language::new();

    language.on_code("12 identity");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_code("7");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn cursor_up_should_move_to_previous_sibling_if_node_has_no_children() {
    // If a node has no children, then moving the cursor up should navigate to
    // the previous sibling.

    let mut language = Language::new();

    language.on_code("a");
    language.on_input(EditorInputEvent::AddSibling);
    language.on_code("c");

    // Verify that the test setup worked.
    assert_eq!(
        language
            .codebase()
            .root()
            .children(language.codebase().nodes())
            .expect_errors()
            .collect_array::<2>()
            .unwrap(),
        ["a", "c"].map(|node| node.to_string()),
    );

    // Actual testing starts here.

    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_code("b");

    assert_eq!(
        language
            .codebase()
            .root()
            .children(language.codebase().nodes())
            .expect_errors()
            .next()
            .unwrap(),
        "ab".to_string(),
    )
}

#[test]
fn moving_cursor_up_at_first_node_should_do_nothing() {
    // If already at the first node, moving to the previous one should do
    // nothing.

    let mut language = Language::new();

    language.on_code("17");
    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_code("2");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn move_cursor_to_parent_node() {
    // If moving the cursor down, and there is no next sibling, the cursor
    // should move to the parent node instead.

    let mut language = Language::new();

    language.on_code("identity dentity");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_input(EditorInputEvent::MoveCursorDown);
    language.on_code("i");

    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());
}

#[test]
fn moving_cursor_down_at_root_node_should_do_nothing() {
    // If already at the last node, moving to the next one should do nothing.

    let mut language = Language::new();

    language.on_code("12");
    language.on_input(EditorInputEvent::MoveCursorDown);
    language.on_code("7");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn moving_cursor_left_at_start_of_node_should_move_to_previous_node() {
    // If the cursor is at the start of a node, then pressing left should move
    // it the end of the previous node.

    let mut language = Language::new();

    language.on_code("12 identity");
    for _ in "identity".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }

    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.on_code("7");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn moving_cursor_right_at_end_of_node_should_move_to_next_node() {
    // If the cursor is at the end of a node, then pressing right should move it
    // the start of the previous node.

    let mut language = Language::new();

    language.on_code("127 dentity");
    language.on_input(EditorInputEvent::MoveCursorUp);

    language.on_input(EditorInputEvent::MoveCursorRight);
    language.on_code("i");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn remove_left_removes_previous_syntax_node_if_empty() {
    // Removing left while cursor is in the leftmost position within the current
    // syntax node, removes the previous syntax node, if that is empty.

    let mut language = Language::new();

    language.on_code(" 127");
    for _ in "127".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }

    // Make sure the test setup worked as expected.
    language
        .codebase()
        .root()
        .node
        .expect_error("127")
        .expect_single_child(language.codebase().nodes())
        .expect_empty();

    // Actual testing starts here.
    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });

    language.codebase().root().node.expect_integer_literal(127);
}

#[test]
fn remove_left_merges_with_previous_syntax_node() {
    // Removing left while cursor is in the leftmost position within the current
    // syntax node, merges the current and the previous syntax node.

    let mut language = Language::new();

    language.on_code("1 27");
    for _ in "27".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn remove_right_removes_next_syntax_node_if_empty() {
    // Removing right while cursor is in the rightmost position within the
    // current syntax node, removes the next syntax node, if that is empty.

    let mut language = Language::new();

    language.on_code("127 ");
    language.on_input(EditorInputEvent::MoveCursorLeft);

    // Make sure the test setup worked as expected.
    language
        .codebase()
        .root()
        .node
        .expect_error("")
        .expect_single_child(language.codebase().nodes())
        .expect_integer_literal(127);

    // Actual testing starts here.
    language.on_input(EditorInputEvent::RemoveRight { whole_node: false });

    assert_eq!(
        language.codebase().root().node,
        &Node::LiteralNumber { value: 127 },
    );
}

#[test]
fn remove_right_merges_with_next_syntax_node() {
    // Removing right while cursor is in the rightmost position within the
    // current syntax node, merges the current and the next syntax node.

    let mut language = Language::new();

    language.on_code("1 27");
    for _ in "27".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }
    language.on_input(EditorInputEvent::MoveCursorLeft);

    language.on_input(EditorInputEvent::RemoveRight { whole_node: false });

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}
