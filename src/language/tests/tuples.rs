use crate::language::{instance::Language, runtime::Value};

#[test]
fn single_field() {
    // It is possible to define a tuple with a single field.

    let mut language = Language::new();

    language.enter_code("127 tuple");
    let output = language.step_until_finished();

    assert_eq!(
        output.map(|value| value.inner),
        Ok(Value::Tuple {
            elements: vec![Value::Integer { value: 127 }]
        })
    );
}

#[test]
fn nested() {
    // It is possible to defined nested tuples.

    let mut language = Language::new();

    language.enter_code("127 tuple tuple");
    let output = language.step_until_finished();

    assert_eq!(
        output.map(|value| value.inner),
        Ok(Value::Tuple {
            elements: vec![Value::Tuple {
                elements: vec![Value::Integer { value: 127 }]
            }]
        })
    );
}
