use itertools::Itertools;

use crate::language::{
    code::{Children, Node},
    editor::EditorInputEvent,
    language::Language,
    packages::{Function, FunctionId, Package},
    runtime::{Effect, Value},
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
fn add_parent_of_node_that_already_has_a_parent() {
    // If a node already has a parent, then adding a parent should add the
    // parent in between them, as a child of the previous parent.

    let mut language = Language::new();

    let package = test_package(&mut language);

    language.on_code("a b_to_c");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_input(EditorInputEvent::AddChildOrParent);
    language.on_code("a_to_b");

    let output = language
        .step_until_finished_and_handle_host_functions(handler(&package));

    assert_eq!(
        output,
        Ok(Value::Opaque {
            id: 2,
            display: "c",
        }),
    );
}

#[test]
fn split_node_if_adding_parent_while_cursor_is_in_the_middle() {
    // If we add a parent while the cursor is in the middle of the current node,
    // we should split the node right there.

    let mut language = Language::new();

    language.on_code("127identity");
    for _ in "identity".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }
    language.on_input(EditorInputEvent::AddChildOrParent);

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn add_sibling() {
    // It is possible to add a sibling to a node.

    let mut language = Language::new();

    language.on_code("a c");
    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.on_input(EditorInputEvent::AddSibling);
    language.on_code("b");

    let root = language.codebase().root();
    assert_eq!(
        root.node,
        &Node::Error {
            node: "c".to_string(),
            children: root.node.to_children(),
        },
    );

    let [a, b] = root
        .children(language.codebase().nodes())
        .collect_array()
        .unwrap();
    assert_eq!(
        a.node,
        &Node::Error {
            node: "a".to_string(),
            children: Children::new([]),
        },
    );
    assert_eq!(
        b.node,
        &Node::Error {
            node: "b".to_string(),
            children: Children::new([]),
        },
    );
}

#[test]
fn add_sibling_to_root_node() {
    // If adding a sibling to the root node, there still needs to be a single
    // root node afterwards. So a new one is created automatically.

    let mut language = Language::new();

    language.on_code("a");
    language.on_input(EditorInputEvent::AddSibling);
    language.on_code("b");

    let root = language.codebase().root();
    assert_eq!(
        root.node,
        &Node::Error {
            node: "".to_string(),
            children: root.node.to_children(),
        },
    );

    let [a, b] = root
        .children(language.codebase().nodes())
        .collect_array()
        .unwrap();
    assert_eq!(
        a.node,
        &Node::Error {
            node: "a".to_string(),
            children: Children::new([]),
        },
    );
    assert_eq!(
        b.node,
        &Node::Error {
            node: "b".to_string(),
            children: Children::new([]),
        },
    );
}

#[test]
fn split_node_if_adding_sibling_while_cursor_is_in_the_middle() {
    // If adding a sibling while the cursor is in the middle of a node, that
    // node should be split.

    let mut language = Language::new();

    language.on_code("ab c");
    for _ in 0..3 {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }
    language.on_input(EditorInputEvent::AddSibling);

    let root = language.codebase().root();
    assert_eq!(
        root.node,
        &Node::Error {
            node: "c".to_string(),
            children: root.node.to_children(),
        },
    );

    let [a, b] = root
        .node
        .to_children()
        .into_iter()
        .map(|hash| language.codebase().nodes().get(&hash))
        .collect_array()
        .unwrap();
    assert_eq!(
        a,
        &Node::Error {
            node: "a".to_string(),
            children: Children::new([]),
        },
    );
    assert_eq!(
        b,
        &Node::Error {
            node: "b".to_string(),
            children: Children::new([]),
        },
    );
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

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum TestFunction {
    A,
    AToB,
    BToC,
}
impl Function for TestFunction {
    fn name(&self) -> &str {
        match self {
            Self::A => "a",
            Self::AToB => "a_to_b",
            Self::BToC => "b_to_c",
        }
    }
}

fn test_package(language: &mut Language) -> Package<TestFunction> {
    language.packages_mut().new_package([
        TestFunction::A,
        TestFunction::AToB,
        TestFunction::BToC,
    ])
}

fn handler(
    package: &Package<TestFunction>,
) -> impl FnMut(&FunctionId, &Value) -> Result<Value, Effect> {
    |id, input| match package.function_by_id(id).unwrap() {
        TestFunction::A => Ok(Value::Opaque {
            id: 0,
            display: "a",
        }),
        TestFunction::AToB => {
            assert_eq!(
                input,
                &Value::Opaque {
                    id: 0,
                    display: "a"
                },
            );

            Ok(Value::Opaque {
                id: 1,
                display: "b",
            })
        }
        TestFunction::BToC => {
            assert_eq!(
                input,
                &Value::Opaque {
                    id: 1,
                    display: "b"
                },
            );

            Ok(Value::Opaque {
                id: 2,
                display: "c",
            })
        }
    }
}
