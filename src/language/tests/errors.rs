use crate::language::{
    code::{CodeError, Codebase, Node},
    compiler::Compiler,
    language::Language,
    packages::{Function, Packages},
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
        language.codebase().errors().get(unresolved.hash()),
        Some(&CodeError::UnresolvedIdentifier { candidates: vec![] }),
    );

    // And it should also result in a runtime error when stepping.
    assert!(language.step().is_error());
}

#[test]
fn fixing_syntax_node_should_remove_error() {
    let mut language = Language::from_code("identit");

    // Make sure that this resulted in an error.
    assert!(
        language
            .codebase()
            .errors()
            .get(language.codebase().root().path.hash())
            .is_some()
    );

    // Once we resolve the error, it should no longer be there.
    language.on_code("y");

    let resolved = language.codebase().root().path;
    assert_eq!(language.codebase().errors().get(resolved.hash()), None);
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

#[test]
fn function_literal_with_too_few_children_is_an_error() {
    // If an `fn` node doesn't have a child, an empty syntax node should be
    // created as a child for it.

    let language = Language::from_code("fn");

    let root = language.codebase().root();

    if let Node::Error { node, .. } = root.node {
        assert_eq!(node, "fn");
    } else {
        panic!();
    }
    assert_eq!(
        language.codebase().errors().get(root.path.hash()),
        Some(&CodeError::TooFewChildren),
    );
}

#[test]
fn function_literal_with_too_many_children_is_an_error() {
    // A function literal should have one child, its body.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    compiler.replace(&compiler.codebase().root().path, "fn", &packages);
    compiler.insert_child(compiler.codebase().root().path, "a", &packages);
    compiler.insert_child(compiler.codebase().root().path, "b", &packages);

    let root = compiler.codebase().root();

    if let Node::Error { node, .. } = root.node {
        assert_eq!(node, "fn");
    } else {
        panic!("Expected error, got `{:?}`", root.node);
    }
    assert_eq!(
        compiler.codebase().errors().get(root.path.hash()),
        Some(&CodeError::TooManyChildren),
    );
}
