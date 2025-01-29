use crate::language::{
    editor::EditorInputEvent,
    instance::Language,
    interpreter::{StepResult, Value},
};

#[test]
fn number_literal_wrong_input() {
    // A number literal is a function that takes `None` and returns `Integer`.
    // So having two in a row means, that the second one does not get the
    // expected input.

    let mut language = Language::new();

    language.enter_code("127");
    language.on_input(EditorInputEvent::SubmitToken);
    language.enter_code("255");

    assert_eq!(
        language.step(),
        StepResult::Finished {
            output: Value::Integer { value: 127 }
        }
    );
    assert_eq!(language.step(), StepResult::Error);
}
