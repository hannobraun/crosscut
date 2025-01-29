use crate::language::{
    editor::EditorInputEvent,
    instance::Language,
    interpreter::{StepResult, Value},
};

#[test]
fn update_on_every_character() {
    // The editor should compile the code on every new character. If the program
    // has finished running, as is the case here, it should also reset the
    // interpreter, so the next step will run the new code.

    let mut language = Language::new();

    language.enter_code("1");
    assert_eq!(language.step_until_finished(), Value::Integer { value: 1 });

    language.enter_code("2");
    assert_eq!(language.step_until_finished(), Value::Integer { value: 12 });
}

#[test]
fn update_after_removing_character() {
    // Removing a character should have an immediate effect on the program, just
    // like adding one.

    let mut language = Language::new();

    language.enter_code("127");
    assert_eq!(
        language.step_until_finished(),
        Value::Integer { value: 127 },
    );

    language.on_input(EditorInputEvent::RemoveLeft);
    assert_eq!(language.step_until_finished(), Value::Integer { value: 12 });

    language.on_input(EditorInputEvent::RemoveLeft);
    assert_eq!(language.step_until_finished(), Value::Integer { value: 1 });
}

#[test]
fn update_after_removing_all_characters() {
    // Removing all characters should have an immediate effect on the program,
    // just like any other edits.

    let mut language = Language::new();

    language.enter_code("1");
    assert_eq!(
        language.step(),
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        }
    );

    language.on_input(EditorInputEvent::RemoveLeft);
    assert_eq!(
        language.step(),
        StepResult::Finished {
            output: Value::None
        }
    );
}

// There is lots of editing behavior that's not tested here, like cursor
// movement and many, many edge cases. This test suite focuses on high-level
// behavior that affects the whole `language` module.
//
// Please refer to the test suite of `EditorInput` for more detailed tests.
