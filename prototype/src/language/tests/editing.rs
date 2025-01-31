use crate::language::{
    editor::EditorInputEvent, instance::Language, runtime::Value,
};

#[test]
fn update_on_every_character() {
    // The editor should compile the code on every new character. If the program
    // has finished running, as is the case here, it should also reset the
    // interpreter, so the next step will run the new code.

    let mut language = Language::without_host();

    language.enter_code("1");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 1 }),
    );

    language.enter_code("2");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 12 }),
    );
}

#[test]
fn update_after_removing_character() {
    // Removing a character should have an immediate effect on the program, just
    // like adding one.

    let mut language = Language::without_host();

    language.enter_code("127");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );

    language.on_input(EditorInputEvent::RemoveLeft);
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 12 }),
    );

    language.on_input(EditorInputEvent::RemoveLeft);
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 1 }),
    );
}

#[test]
fn update_after_removing_all_characters() {
    // Removing all characters should have an immediate effect on the program,
    // just like any other edits.

    let mut language = Language::without_host();

    language.enter_code("1");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 1 }),
    );

    language.on_input(EditorInputEvent::RemoveLeft);
    assert_eq!(language.step_until_finished(), Ok(Value::None));
}

#[test]
fn split_node_if_submitting_while_cursor_is_in_the_middle() {
    // If we submit the token we currently edit, while the cursor is in the
    // middle of it, we should split the token right there.

    let mut language = Language::without_host();

    language.enter_code("127identity");
    for _ in "identity".chars() {
        language.on_input(EditorInputEvent::MoveCursorLeft);
    }
    language.on_input(EditorInputEvent::SubmitNode);

    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_up_should_navigate_to_previous_node() {
    // It is possible to navigate to the previous node in the editor.

    let mut language = Language::without_host();

    language.enter_code("12 identity");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.enter_code("7");

    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_up_at_first_node_should_do_nothing() {
    // If already at the first node, moving to the previous one should do
    // nothing.

    let mut language = Language::without_host();

    language.enter_code("17");
    language.on_input(EditorInputEvent::MoveCursorLeft);
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.enter_code("2");

    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn moving_cursor_down_should_navigate_to_next_node() {
    // It is possible to navigate to the next node in the editor.

    let mut language = Language::without_host();

    language.enter_code("identity identit");
    language.on_input(EditorInputEvent::MoveCursorUp);
    language.on_input(EditorInputEvent::MoveCursorDown);
    language.enter_code("y");

    assert_eq!(language.step_until_finished(), Ok(Value::None));
}

// There is lots of editing behavior that's not tested here, as this test suite
// focuses on high-level behavior that affects the whole `language` module.
//
// Please refer to the test suite of `EditorInputBuffer` for more detailed
// tests.
