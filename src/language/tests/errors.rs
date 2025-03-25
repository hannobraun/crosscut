use crate::language::{
    code::{CodeError, Expression, IntrinsicFunction, Type},
    language::Language,
    packages::Function,
    runtime::{Effect, RuntimeState, Value},
};

#[test]
fn number_literal_wrong_input() {
    // A number literal is a function that takes `None` and returns `Integer`.
    // So having two in a row means, that the second one does not get the
    // expected input.

    // The compiler doesn't do type checking at this point, so it doesn't know
    // that the second number literal gets an invalid input.
    let mut language = Language::from_code("127 255");

    let invalid = language.codebase().root().path;

    assert!(language.step().is_running());
    assert_eq!(
        language.step(),
        &RuntimeState::Effect {
            effect: Effect::UnexpectedInput {
                expected: Type::Nothing,
                actual: Value::Integer { value: 127 },
            },
            path: invalid,
        },
    );
}

#[test]
fn unresolved_syntax_node() {
    // If a syntax node does not refer to a known function, that should result
    // in an error.

    let mut language = Language::from_code("identit");

    // The error should be registered in `Codebase`.
    let unresolved = language.codebase().root().path;
    assert_eq!(
        language.codebase().errors().get(&unresolved),
        Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
    );

    // And it should also result in a runtime error when stepping.
    assert!(language.step().is_error());

    // Once we resolve the error, it should no longer be there.
    language.on_code("y");

    let resolved = language.codebase().root().path;
    assert_eq!(language.codebase().errors().get(&resolved), None);
    assert_eq!(language.step_until_finished().unwrap(), Value::Nothing);
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

    let mut language = Language::new();

    let mut package = language.packages_mut().new_package();
    let identity = package.add_function(Identity);

    language.on_code("identity");

    let unresolved = language.codebase().root().path;
    assert_eq!(
        language.codebase().errors().get(&unresolved),
        Some(&CodeError::UnresolvedIdentifier {
            candidates: vec![
                Expression::HostFunction { id: identity },
                Expression::IntrinsicFunction {
                    intrinsic: IntrinsicFunction::Identity,
                },
            ]
        }),
    );
}

#[test]
fn evaluate_code_up_until_an_error() {
    // Despite the presence of an error in the code, any valid code leading up
    // to it should still get evaluated.

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Ping;
    impl Function for Ping {
        fn name(&self) -> &str {
            "ping"
        }
    }

    let mut language = Language::new();

    let mut package = language.packages_mut().new_package();
    package.add_function(Ping);

    language.on_code("ping unresolved");

    assert!(matches!(
        language.step(),
        RuntimeState::Effect {
            effect: Effect::ApplyHostFunction { .. },
            ..
        },
    ));
    language.provide_host_function_output(Value::Nothing);

    assert!(matches!(language.step(), RuntimeState::Error { .. }));
}

#[test]
fn do_not_step_beyond_errors() {
    // If there's an error in the code, the interpreter should never step beyond
    // that, if it encounters it.

    let mut language = Language::from_code("unresolved");

    assert!(language.step().is_error());
    assert!(language.step().is_error());
}
