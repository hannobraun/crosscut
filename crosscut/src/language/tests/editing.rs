use crate::language::{
    editor::EditorInput,
    language::Language,
    runtime::{Effect, RuntimeState, Value},
};

#[test]
fn update_on_every_character() {
    // The editor should compile the code on every new character. If the program
    // has finished running, as is the case here, it should also reset the
    // interpreter, so the next step will run the new code.

    let mut language = Language::new();

    language.code("1");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 1 },
    );

    language.code("2");
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

    language.code("127");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );

    language.on_editor_input(EditorInput::RemoveLeft { whole_node: false });
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 12 },
    );

    language.on_editor_input(EditorInput::RemoveLeft { whole_node: false });
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

    language.code("1");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 1 },
    );

    language.on_editor_input(EditorInput::RemoveLeft { whole_node: false });
    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());
}

#[test]
fn keep_state_on_update_to_code_after_current_step() {
    // An update of code that comes after the current step, keeps all the
    // runtime state.

    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("fn")
        .down()
        .code("i")
        .down()
        .code("apply")
        .down()
        .code("notify_test")
        .down()
        .code("i")
        .down()
        .code("apply")
        .down()
        .code("self")
        .down()
        .code("apply")
        .down()
        .code("+")
        .down()
        .code("tuple")
        .down()
        .code("i")
        .down()
        .code("1")
        .down()
        .down() // navigate past tuple
        .down() // navigate past function body
        .code("0");

    wait_for(&mut language, 0);
    wait_for(&mut language, 1);
    wait_for(&mut language, 2);

    language
        .up()
        .up()
        .up()
        .up()
        .up()
        .up()
        .remove_left()
        .code("-");

    wait_for(&mut language, 1);

    fn wait_for(language: &mut Language, expected_value: i32) {
        for _ in 0..1024 {
            if let RuntimeState::Effect {
                effect: Effect::ApplyProvidedFunction { name, input },
                ..
            } = language.step()
            {
                assert_eq!(name, "notify_test");

                let Value::Integer { value } = input else {
                    panic!("Expected integer, got `{input:?}`.");
                };

                assert_eq!(value, &expected_value);

                language.exit_from_provided_function(Value::nothing());

                return;
            }
        }

        panic!("Expected call to provided function was not received.");
    }
}
