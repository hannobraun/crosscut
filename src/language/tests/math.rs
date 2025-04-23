use crate::language::{language::Language, runtime::Value};

#[test]
fn add() {
    let mut language = Language::from_code_postfix("1\n2 tuple +");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Integer { value: 3 },
    );
}
