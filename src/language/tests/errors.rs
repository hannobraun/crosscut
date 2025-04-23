use crate::language::{
    code::{CodeError, Node},
    language::Language,
    packages::Function,
    runtime::{RuntimeState, Value},
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
    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());
}

#[test]
fn children_of_error_should_not_be_evaluated() {
    // Most of the time, it would make sense to evaluate any valid code, until
    // an error is encountered. But some of the time, the erroneous node might
    // be intended as a function literal. And then just starting to execute the
    // erroneous function where it's defined, would be wild and unexpected.

    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Ping;
    impl Function for Ping {
        fn name(&self) -> &str {
            "ping"
        }
    }

    let mut language = Language::postfix();
    language.packages_mut().new_package([Ping]);

    language.on_code("ping unresolved");

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
    // An `fn` node is expected to have one child, its body. If it has fewer
    // than that, that's an error.

    expect_error_because_of_too_few_children("fn");
    expect_error_because_of_too_few_children("a fn");

    fn expect_error_because_of_too_few_children(code: &str) {
        let language = Language::from_code(code);

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
}

#[test]
fn function_literal_with_too_many_children_is_an_error() {
    // An `fn` node is expected to have one child, its body. If it has more than
    // that, that's an error.

    let language = Language::from_code("a\nb\nc fn");

    let root = language.codebase().root();

    if let Node::Error { node, .. } = root.node {
        assert_eq!(node, "fn");
    } else {
        panic!("Expected error, got `{:?}`", root.node);
    }
    assert_eq!(
        language.codebase().errors().get(root.path.hash()),
        Some(&CodeError::TooManyChildren),
    );
}
