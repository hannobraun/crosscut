use crate::language::{
    code::CodeError,
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

    language.enter_code("identit");

    // The error should be registered in `Codebase`.
    let unresolved = language.codebase().nodes().next().unwrap().location;
    assert_eq!(
        language.codebase().error_at(&unresolved),
        Some(&CodeError::UnresolvedIdentifier),
    );

    // And it should also result in a runtime error when stepping.
    assert_eq!(language.step(), StepResult::Error);

    // Once we resolve the error, it should no longer be there.
    language.enter_code("y");

    let resolved = language.codebase().nodes().next().unwrap().location;
    assert_eq!(language.codebase().error_at(&resolved), None);
    assert_eq!(language.step_until_finished(), Ok(Value::None));
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
