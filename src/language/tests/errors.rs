use crate::language::{
    code::{CodeError, Expression, IntrinsicFunction},
    instance::Language,
    packages::{FunctionId, Package},
    runtime::{StepResult, Value},
};

#[test]
fn number_literal_wrong_input() {
    // A number literal is a function that takes `None` and returns `Integer`.
    // So having two in a row means, that the second one does not get the
    // expected input.

    let mut language = Language::without_package();

    language.enter_code("127 255");

    assert_eq!(
        language.step(),
        StepResult::FunctionApplied {
            output: Value::Integer { value: 127 }
        },
    );
    assert_eq!(language.step(), StepResult::Error);
}

#[test]
fn unresolved_syntax_node() {
    // If a syntax node does not refer to a known function, that should result
    // in an error.

    let mut language = Language::without_package();

    language.enter_code("identit");

    // The error should be registered in `Codebase`.
    let unresolved = language.codebase().entry_to_root().next().unwrap().path;
    assert_eq!(
        language.codebase().error_at(&unresolved),
        Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
    );

    // And it should also result in a runtime error when stepping.
    assert_eq!(language.step(), StepResult::Error);

    // Once we resolve the error, it should no longer be there.
    language.enter_code("y");

    let resolved = language.codebase().entry_to_root().next().unwrap().path;
    assert_eq!(language.codebase().error_at(&resolved), None);
    assert_eq!(language.step_until_finished(), Ok(Value::None));
}

#[test]
fn syntax_node_that_could_resolve_to_multiple_functions_is_unresolved() {
    // If a syntax node could resolve to multiple functions, it should remain
    // unresolved, and an error should be shown.

    let mut package = Package::new();
    package.function(FunctionId { id: 0 }, "identity");

    let mut language = Language::with_package(package);

    language.enter_code("identity");

    let unresolved = language.codebase().entry_to_root().next().unwrap().path;
    assert_eq!(
        language.codebase().error_at(&unresolved),
        Some(&CodeError::UnresolvedIdentifier {
            candidates: vec![
                Expression::HostFunction {
                    id: FunctionId { id: 0 }
                },
                Expression::IntrinsicFunction {
                    function: IntrinsicFunction::Identity
                }
            ]
        }),
    );
}

#[test]
fn do_not_step_beyond_errors() {
    // If there's an error in the code, the interpreter should never step beyond
    // that, if it encounters it.

    let mut language = Language::without_package();

    language.enter_code("error");

    assert_eq!(language.step(), StepResult::Error);
    assert_eq!(language.step(), StepResult::Error);
}
