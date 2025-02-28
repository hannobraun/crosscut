use crate::language::{
    code::NodePath,
    instance::Language,
    runtime::{Effect, Value},
};

#[test]
fn define_and_evaluate() {
    // It is possible to define a function using a function literal, return that
    // function from the program, then tell the language to evaluate it.

    let mut language = Language::new();

    language.enter_code("127 fn");
    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.push_context(path, Value::Nothing);
    assert_eq!(
        language.step_until_finished(),
        Ok(Value::Integer { value: 127 }),
    );
}

#[test]
fn self_recursion() {
    // A function can recurse using the `self` keyword.

    let mut language = Language::new();

    language.enter_code("identity self fn");
    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.push_context(path, Value::Integer { value: 127 });

    // To verify we are actually recursing, we need to see at least two values.
    // But evaluating `self` could result in its own iterator step. We need to
    // account for that.
    for _ in 0..3 {
        assert_eq!(
            language.step().active_value(),
            Some(&Value::Integer { value: 127 }),
        );
    }
}

#[test]
fn empty_function() {
    // If an `fn` node doesn't have a child, an empty syntax node should be
    // created as a child for it.

    let mut language = Language::new();

    language.enter_code("fn");
    let path = match language.step_until_finished().into_function_body() {
        Ok(path) => path,
        output => {
            panic!("Unexpected output: {output:?}");
        }
    };

    language.push_context(path, Value::Nothing);
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
