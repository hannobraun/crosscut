use crate::language::{
    code::{CodeError, NodePath},
    instance::Language,
    runtime::{Effect, StepResult, Value, ValueWithSource},
};

#[test]
fn define_and_evaluate() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::without_package();

    language.enter_code("127 fn");
    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.evaluate(path, Value::Nothing);
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn self_recursion() {
    // A function can recurse using the `self` keyword.

    let mut language = Language::without_package();

    language.enter_code("127 self fn");
    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.evaluate(path, Value::Nothing);

    assert_eq!(
        language.step().active_value(),
        Some(Value::Integer { value: 127 }),
    );
    assert_eq!(language.step(), StepResult::Recursing);
    assert_eq!(
        language.step().active_value(),
        Some(Value::Integer { value: 127 }),
    );
}

#[test]
fn empty_function() {
    // An `fn` token that doesn't follow an expression would create a function
    // without a body. This is an error.

    let mut language = Language::without_package();

    language.enter_code("fn");

    let bare_fn = language.codebase().root().path;
    assert_eq!(
        language.codebase().error_at(&bare_fn),
        Some(&CodeError::FunctionWithoutBody),
    );

    assert!(matches!(language.step(), StepResult::Error { .. }));
}

pub trait IntoFunctionBody {
    fn into_function_body(self) -> Result<NodePath, Self>
    where
        Self: Sized;
}

impl IntoFunctionBody for Result<ValueWithSource, Effect> {
    fn into_function_body(self) -> Result<NodePath, Self> {
        self.map_err(Err)
            .and_then(|value| value.into_function_body().map_err(Ok))
    }
}
