use crate::language::{
    instance::Language,
    interpreter::{StepResult, Value},
};

#[test]
fn number_literal() {
    // A number literal is a function that takes `None` and returns the number
    // it represents.

    let mut language = Language::new();

    language.enter_code("127");
    assert_eq!(
        language.step(),
        StepResult::Finished {
            output: Value::Integer { value: 127 }
        },
    );
}

#[test]
fn identity_none() {
    // The `identity` function takes any argument and returns it unchanged. The
    // initial value is `None`, so an `identity` with nothing else around,
    // should return that.

    let mut language = Language::new();

    language.enter_code("identity");
    assert_eq!(
        language.step(),
        StepResult::Finished {
            output: Value::None
        },
    );
}
