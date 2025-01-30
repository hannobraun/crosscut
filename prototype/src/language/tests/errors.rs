use crate::language::{
    instance::Language,
    runtime::{StepResult, Value},
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

#[test]
fn unresolved_identifier() {
    // If an identifier does not refer to a known function, that should result
    // in an error.

    let mut language = Language::without_host();

    language.enter_code("unknown");

    assert_eq!(language.step(), StepResult::Error);
}

#[test]
fn do_not_step_beyond_errors() {
    // If there's an error in the code, the interpreter should never step beyond
    // that, if it encounters it.

    let mut language = Language::without_host();

    language.enter_code("error");

    assert_eq!(language.step(), StepResult::Error);
    assert_eq!(language.step(), StepResult::Error);
}
