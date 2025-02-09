use crate::language::{
    code::CodeError,
    instance::Language,
    runtime::{StepResult, Value},
};

#[test]
fn define_and_evaluate_function() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::without_package();

    language.enter_code("127 fn");
    let path = match language
        .step_until_finished()
        .map_err(|effect| (Some(effect), None))
        .and_then(|value| {
            value
                .into_function_body()
                .map_err(|value| (None, Some(value)))
        }) {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.evaluate(path);
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn function_without_body() {
    // An `fn` token that doesn't follow an expression would create a function
    // without a body. This is an error.

    let mut language = Language::without_package();

    language.enter_code("fn");

    let bare_fn = language.codebase().root().path;
    assert_eq!(
        language.codebase().error_at(&bare_fn),
        Some(&CodeError::FunctionWithoutBody),
    );

    assert_eq!(language.step(), StepResult::Error);
}
