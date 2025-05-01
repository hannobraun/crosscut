use crate::language::{
    code::CodeError, language::Language, runtime::Value,
    tests::infra::LocatedNodeExt,
};

#[test]
fn unresolved_syntax_node() {
    // If a syntax node does not refer to a known function, that should result
    // in an error.

    let mut language = Language::new();
    language.code("identit");

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
    let mut language = Language::new();
    language.code("apply").down().code("identit");

    // Make sure that this resulted in an error.
    let root = language.codebase().root();
    let [invalid, _] = root.expect_children(language.codebase().nodes());
    assert!(
        language
            .codebase()
            .errors()
            .get(invalid.path.hash())
            .is_some()
    );

    // Once we resolve the error, it should no longer be there.
    language.code("y");

    let resolved = language.codebase().root().path;
    assert_eq!(language.codebase().errors().get(resolved.hash()), None);
    assert_eq!(language.step_until_finished().unwrap(), Value::nothing());
}

#[test]
fn do_not_step_beyond_errors() {
    // If there's an error in the code, the interpreter should never step beyond
    // that, if it encounters it.

    let mut language = Language::new();
    language.code("unresolved");

    assert!(language.step().is_error());
    assert!(language.step().is_error());
}
