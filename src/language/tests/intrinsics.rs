use crate::language::{language::Language, runtime::Value};

#[test]
fn drop() {
    // The `drop` function takes any argument and returns `nothing`.

    let mut language = Language::new();
    language.code("drop 127");

    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());
}

#[test]
fn number_literal() {
    // A number literal is a function that takes `nothing` and returns the
    // number it represents.

    let mut language = Language::new();
    language.code("127");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}

#[test]
fn identity() {
    // The `identity` function takes any argument and returns it unchanged.

    let mut language = Language::new();
    language.code("identity");
    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());

    let mut language = Language::new();
    language.code("identity 127");
    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 127 },
    );
}
