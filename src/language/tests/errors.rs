use crate::language::{
    code::CodeError,
    language::Language,
    packages::Function,
    runtime::{Effect, RuntimeState, Value},
};

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
            effect: Effect::ProvidedFunction { .. },
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
