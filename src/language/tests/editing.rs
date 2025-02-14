use itertools::Itertools;

use crate::language::{
    code::Node,
    editor::EditorInputEvent,
    instance::Language,
    packages::{Function, FunctionId, Package},
    runtime::Value,
};

#[test]
fn update_on_every_character() {
    // The editor should compile the code on every new character. If the program
    // has finished running, as is the case here, it should also reset the
    // interpreter, so the next step will run the new code.

    let mut language = Language::without_package();

    language.enter_code("1");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 1 }),
    );

    language.enter_code("2");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 12 }),
    );
}

#[test]
fn update_after_removing_character() {
    // Removing a character should have an immediate effect on the program, just
    // like adding one.

    let mut language = Language::without_package();

    language.enter_code("127");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 12 }),
    );

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 1 }),
    );
}

#[test]
fn update_after_removing_all_characters() {
    // Removing all characters should have an immediate effect on the program,
    // just like any other edits.

    let mut language = Language::without_package();

    language.enter_code("1");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 1 }),
    );

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Nothing),
    );
}

#[test]
fn submitting_the_node_should_insert_a_new_one_after_the_current_one() {
    // When submitting a node, a new one should be inserted after the one we
    // just submitted.

    enum TestFunction {
        Zero,
        IfZeroThen127,
    }
    impl Function for TestFunction {
        fn from_id(FunctionId { id }: FunctionId) -> Option<Self> {
            match id {
                0 => Some(Self::Zero),
                1 => Some(Self::IfZeroThen127),
                _ => None,
            }
        }
        fn id(&self) -> FunctionId {
            let id = match self {
                Self::Zero => 0,
                Self::IfZeroThen127 => 1,
            };

            FunctionId { id }
        }
        fn name(&self) -> &str {
            match self {
                Self::Zero => "zero",
                Self::IfZeroThen127 => "if_zero_then_127",
            }
        }
    }

    let mut package = Package::new();
    package.function(TestFunction::Zero);
    package.function(TestFunction::IfZeroThen127);

    let mut language = Language::with_package(package);

    language.enter_code("255 if_zero_then_127");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_input(EditorInputEvent::SubmitNode);
    language.enter_code("zero");

    let output =
        language.step_until_finished_and_handle_host_functions(|id, input| {
            match TestFunction::from_verified_id(id) {
                TestFunction::Zero => Ok(Value::Integer { value: 0 }),
                TestFunction::IfZeroThen127 => {
                    if let Value::Integer { value: 0 } = input {
                        Ok(Value::Integer { value: 127 })
                    } else {
                        Ok(input)
                    }
                }
            }
        });

    assert_eq!(
        output.map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn split_node_if_submitting_while_cursor_is_in_the_middle() {
    // If we submit the node we're currently editing, while the cursor is in the
    // middle of it, we should split the node right there.

    let mut language = Language::without_package();

    language.enter_code("127identity");
    for _ in "identity".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }
    language.on_input(EditorInputEvent::SubmitNode);

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_up_should_navigate_to_previous_node() {
    // It is possible to navigate to the previous node in the editor.

    let mut language = Language::without_package();

    language.enter_code("12 identity");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.enter_code("7");

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_up_at_first_node_should_do_nothing() {
    // If already at the first node, moving to the previous one should do
    // nothing.

    let mut language = Language::without_package();

    language.enter_code("17");
    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.enter_code("2");

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_down_should_navigate_to_next_node() {
    // It is possible to navigate to the next node in the editor.

    let mut language = Language::without_package();

    language.enter_code("identity dentity");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_input(EditorInputEvent::MoveCursorDown);
    language.enter_code("i");

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Nothing),
    );
}

#[test]
fn moving_cursor_down_at_last_node_should_do_nothing() {
    // If already at the last node, moving to the next one should do nothing.

    let mut language = Language::without_package();

    language.enter_code("12");
    language.on_input(EditorInputEvent::MoveCursorDown);
    language.enter_code("7");

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_left_at_start_of_node_should_move_to_previous_node() {
    // If the cursor is at the start of a node, then pressing left should move
    // it the end of the previous node.

    let mut language = Language::without_package();

    language.enter_code("12 identity");
    for _ in "identity".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }

    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.enter_code("7");

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_right_at_end_of_node_should_move_to_next_node() {
    // If the cursor is at the end of a node, then pressing right should move it
    // the start of the previous node.

    let mut language = Language::without_package();

    language.enter_code("127 dentity");
    language.on_input(EditorInputEvent::MoveCursorUp);

    language.on_input(EditorInputEvent::MoveCursorRight);
    language.enter_code("i");

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn remove_left_removes_previous_syntax_node_if_empty() {
    // Removing left while cursor is in the leftmost position within the current
    // syntax node, removes the previous syntax node, if that is empty.

    let mut language = Language::without_package();

    language.enter_code(" 127");
    for _ in "127".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }

    // Make sure the test setup worked as expected.
    let (empty, literal) =
        language.codebase().leaf_to_root().collect_tuple().unwrap();
    assert_eq!(empty.node, &Node::Empty { child: None });
    assert_eq!(
        literal.node,
        &Node::integer_literal(127, Some(*empty.path.hash())),
    );

    // Actual testing starts here.
    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });

    let (literal,) =
        language.codebase().leaf_to_root().collect_tuple().unwrap();
    assert_eq!(literal.node, &Node::integer_literal(127, None));
}

#[test]
fn remove_left_merges_with_previous_syntax_node() {
    // Removing left while cursor is in the leftmost position within the current
    // syntax node, merges the current and the previous syntax node.

    let mut language = Language::without_package();

    language.enter_code("1 27");
    for _ in "27".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }

    language.on_input(EditorInputEvent::RemoveLeft { whole_node: false });

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn remove_right_removes_next_syntax_node_if_empty() {
    // Removing right while cursor is in the rightmost position within the
    // current syntax node, removes the next syntax node, if that is empty.

    let mut language = Language::without_package();

    language.enter_code("127 ");
    language.on_input(EditorInputEvent::MoveCursorLeft);

    // Make sure the test setup worked as expected.
    let (literal, empty) =
        language.codebase().leaf_to_root().collect_tuple().unwrap();
    assert_eq!(literal.node, &Node::integer_literal(127, None));
    assert_eq!(
        empty.node,
        &Node::Empty {
            child: Some(*literal.path.hash()),
        },
    );

    // Actual testing starts here.
    language.on_input(EditorInputEvent::RemoveRight { whole_node: false });

    let (literal,) =
        language.codebase().leaf_to_root().collect_tuple().unwrap();
    assert_eq!(literal.node, &Node::integer_literal(127, None));
}

#[test]
fn remove_right_merges_with_next_syntax_node() {
    // Removing right while cursor is in the rightmost position within the
    // current syntax node, merges the current and the next syntax node.

    let mut language = Language::without_package();

    language.enter_code("1 27");
    for _ in "27".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }
    language.on_input(EditorInputEvent::MoveCursorLeft);

    language.on_input(EditorInputEvent::RemoveRight { whole_node: false });

    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

// There is lots of editing behavior that's not tested here, as this test suite
// focuses on high-level behavior that affects the whole `language` module.
//
// Please refer to the test suite of `EditorInputBuffer` for more detailed
// tests.
