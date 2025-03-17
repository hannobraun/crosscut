use crate::language::{
    language::Language, runtime::Value,
    tests::infra::StepUntilFinishedResultExt,
};

#[test]
fn single_field() {
    // It is possible to define a tuple with a single field.

    let mut language = Language::new();

    language.on_code("127 tuple");

    assert_eq!(
        language.step_until_finished().expect_value(),
        Value::Tuple {
            elements: vec![Value::Integer { value: 127 }],
        },
    );
}

#[test]
fn nested() {
    // It is possible to defined nested tuples.

    let mut language = Language::new();

    language.on_code("127 tuple tuple");

    assert_eq!(
        language.step_until_finished().expect_value(),
        Value::Tuple {
            elements: vec![Value::Tuple {
                elements: vec![Value::Integer { value: 127 }],
            }],
        },
    );
}

#[test]
fn multi_field() {
    // It is possible to define a tuple with multiple fields.

    let mut language = Language::from_code("127\n255 tuple");

    assert_eq!(
        language.step_until_finished().expect_value(),
        Value::Tuple {
            elements: vec![
                Value::Integer { value: 127 },
                Value::Integer { value: 255 },
            ],
        },
    );
}
