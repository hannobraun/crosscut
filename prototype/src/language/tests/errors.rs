use crate::language::{
    instance::Language,
    interpreter::{StepResult, Value},
};

#[test]
fn number_literal_wrong_input() {
    // A number literal is a function that takes `None` and returns `Integer`.
    // So having two in a row means, that the second one does not get the
    // expected input.

    let mut language = Language::without_host();

    language.enter_code("127 255");

    assert_eq!(
        language.step(),
        StepResult::FunctionApplied {
            output: Value::Integer { value: 127 }
        },
    );
    assert_eq!(language.step(), StepResult::Error);
}
