use crate::language::{
    code::NodePath,
    language::Language,
    runtime::{Effect, Value},
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

    let mut language = Language::from_code("self fn");
    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.call_function(path, Value::Nothing);

    // To verify we are actually recursing, we need to see at least two values.
    // But evaluating `self` could result in its own iterator step. We need to
    // account for that.
    for _ in 0..3 {
        assert_eq!(language.step().active_value(), Some(&Value::Nothing));
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
