use crate::language::{
    code::NodePath,
    language::Language,
    runtime::{Effect, RuntimeState, Value},
};

#[test]
fn define_and_evaluate() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::from_code("127 fn eval");
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn self_recursion() {
    // A function can recurse using the `self` keyword.

    let mut language = Language::from_code("self fn eval");

    // This is a rather large number of steps, given the length of the program.
    // Should be proof enough, that it's recursing.
    for _ in 0..1024 {
        assert!(matches!(language.step(), RuntimeState::Running { .. }));
    }
}

#[test]
fn empty_function() {
    // If an `fn` node doesn't have a child, an empty syntax node should be
    // created as a child for it.

    let mut language = Language::from_code("fn eval");
    assert_eq!(language.step_until_finished(), Ok(Value::Nothing));
}

pub trait IntoFunctionBody {
    fn into_function_body(self) -> Result<NodePath, Self>
    where
        Self: Sized;
}

impl IntoFunctionBody for Result<Value, Effect> {
    fn into_function_body(self) -> Result<NodePath, Self> {
        self.map_err(Err)
            .and_then(|value| value.into_function_body().map_err(Ok))
    }
}
