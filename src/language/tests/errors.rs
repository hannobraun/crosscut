use crate::language::{
    code::{CodeError, Expression, IntrinsicFunction, Type},
    instance::Language,
    packages::{Function, FunctionId, Package},
    runtime::{Effect, RuntimeState, Value},
};

#[test]
fn number_literal_wrong_input() {
    // A number literal is a function that takes `None` and returns `Integer`.
    // So having two in a row means, that the second one does not get the
    // expected input.

    let mut language = Language::new();

    language.enter_code("127 255");

    assert_eq!(
        language.step().active_value(),
        Some(Value::Integer { value: 127 }),
    );
    assert!(matches!(language.step(), RuntimeState::Effect { .. }));
}

#[test]
fn unresolved_syntax_node() {
    // If a syntax node does not refer to a known function, that should result
    // in an error.

    let mut language = Language::new();

    language.enter_code("identit");

    // The error should be registered in `Codebase`.
    let unresolved = language.codebase().root().path;
    assert_eq!(
        language.codebase().error_at(&unresolved),
        Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
    );

    // And it should also result in a runtime error when stepping.
    assert!(matches!(language.step(), RuntimeState::Error { .. }));

    // Once we resolve the error, it should no longer be there.
    language.enter_code("y");

    let resolved = language.codebase().root().path;
    assert_eq!(language.codebase().error_at(&resolved), None);
    assert_eq!(
        language.step_until_finished().map(|value| value.inner),
        Ok(Value::Nothing),
    );
}

#[test]
fn syntax_node_that_could_resolve_to_multiple_functions_is_unresolved() {
    // If a syntax node could resolve to multiple functions, it should remain
    // unresolved, and an error should be shown.

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Identity;
    impl Function for Identity {
        fn name(&self) -> &str {
            "identity"
        }
    }

    let package = Package::new().with_function(Identity);

    let mut language = Language::new();
    language.with_package(&package);

    language.enter_code("identity");

    let unresolved = language.codebase().root().path;
    assert_eq!(
        language.codebase().error_at(&unresolved),
        Some(&CodeError::UnresolvedIdentifier {
            candidates: vec![
                Expression::HostFunction {
                    id: FunctionId { id: 0 }
                },
                Expression::IntrinsicFunction {
                    intrinsic: IntrinsicFunction::Identity
                }
            ]
        }),
    );
}

#[test]
fn do_not_step_beyond_errors() {
    // If there's an error in the code, the interpreter should never step beyond
    // that, if it encounters it.

    let mut language = Language::new();

    language.enter_code("error");

    assert!(matches!(language.step(), RuntimeState::Error { .. }));
    assert!(matches!(language.step(), RuntimeState::Error { .. }));
}

#[test]
fn pure_runtime_error_should_result_in_error_state() {
    // Some errors are not known at compile-time and are only encountered at
    // runtime. Such an error should still be reported by the evaluator.

    let mut language = Language::new();

    // The compiler doesn't do type checking at this point, so it doesn't know
    // that the second number literal gets an invalid input.
    language.enter_code("127 127");

    assert_eq!(
        language.step().active_value(),
        Some(Value::Integer { value: 127 }),
    );
    assert!(matches!(language.step(), RuntimeState::Effect { .. }));

    let invalid = language.codebase().root().path;
    assert_eq!(
        language.evaluator().state(),
        &RuntimeState::Effect {
            effect: Effect::UnexpectedInput {
                expected: Type::Nothing,
                actual: Value::Integer { value: 127 },
            },
            path: invalid,
        },
    );
}
