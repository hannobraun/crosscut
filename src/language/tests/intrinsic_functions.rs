use crate::language::{instance::Language, runtime::Value};

#[test]
fn number_literal() {
    // A number literal is a function that takes `None` and returns the number
    // it represents.

    let mut language = Language::without_package();

    language.enter_code("127");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn identity_none() {
    // The `identity` function takes any argument and returns it unchanged. The
    // initial value is `None`, so an `identity` with nothing else around,
    // should return that.

    let mut language = Language::without_package();

    language.enter_code("identity");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Nothing),
    );
}

#[test]
fn identity_integer() {
    // The `identity` function takes any argument and returns it unchanged. This
    // works with integers, as it does with any other value.

    let mut language = Language::without_package();

    language.enter_code("127 identity");
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}
