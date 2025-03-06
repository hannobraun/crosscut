use crate::language::{language::Language, runtime::Value};

#[test]
fn eval() {
    // The `eval` function takes a function argument and evaluates that.
    //
    // So far, the `eval` function can only pass `nothing` to the evaluated
    // function. Eventually, it should be able to pass any argument.

    let mut language = Language::from_code("127 fn eval");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn number_literal() {
    // A number literal is a function that takes `nothing` and returns the
    // number it represents.

    let mut language = Language::from_code("127");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn identity_none() {
    // The `identity` function takes any argument and returns it unchanged.

    let mut language = Language::from_code("identity");
    assert_eq!(language.step_until_finished(), Ok(Value::Nothing));

    let mut language = Language::from_code("127 identity");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}
