use crate::language::{language::Language, runtime::Value};

#[test]
fn add() {
    let mut language = Language::new();
    language
        .code("apply")
        .down()
        .code("+")
        .down()
        .code("tuple 1\n2");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 3 },
    );
}
