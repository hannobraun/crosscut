use crate::language::{language::Language, runtime::Value};

#[test]
fn single_field() {
    // It is possible to define a tuple with a single field.

    let mut language = Language::new();
    language.code("tuple 127");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Tuple {
            values: vec![Value::Integer { value: 127 }],
        },
    );
}

#[test]
fn nested() {
    // It is possible to defined nested tuples.

    let mut language = Language::new();
    language.code("tuple tuple 127");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Tuple {
            values: vec![Value::Tuple {
                values: vec![Value::Integer { value: 127 }],
            }],
        },
    );
}

#[test]
fn multi_field() {
    // It is possible to define a tuple with multiple fields.

    let mut language = Language::new();
    language.code("tuple 127\n255");

    assert_eq!(
        language.step_until_finished().unwrap(),
        Value::Tuple {
            values: vec![
                Value::Integer { value: 127 },
                Value::Integer { value: 255 },
            ],
        },
    );
}
