use itertools::Itertools;

use crate::lang::{
    code::{Code, CodeError},
    compiler::tests::infra::compile_all,
    host::Host,
};

#[test]
fn integer_literal_larger_than_32_bits_is_an_error() {
    // If an integer literal is larger than 32 bits, that is an error.

    let host = Host::empty();

    let mut code = Code::default();
    compile_all("4294967295", &host, &mut code);
    let i = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .next()
        .unwrap();
    assert_eq!(code.errors.get(i), None);

    let mut code = Code::default();
    compile_all("4294967296", &host, &mut code);
    let i = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .next()
        .unwrap();
    assert_eq!(code.errors.get(i), Some(&CodeError::IntegerOverflow));
}

#[test]
fn code_after_expression_is_an_error() {
    // An expression returns a value. That value can be returned when the
    // program finishes, or it can be used as the argument of a function call.
    //
    // Either way, any code that comes after an expression makes no sense, and
    // is an error.

    let host = Host::empty();
    let mut code = Code::default();

    compile_all("1 2", &host, &mut code);

    let (a, b) = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .collect_tuple()
        .unwrap();

    assert_eq!(code.errors.get(a), None);
    assert_eq!(code.errors.get(b), Some(&CodeError::UnexpectedToken));
}

#[test]
fn unresolved_identifier_is_an_error() {
    // An identifier that does not refer to a function is an error.

    let host = Host::empty();
    let mut code = Code::default();

    compile_all("f 1", &host, &mut code);

    let f = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .next()
        .unwrap();
    assert_eq!(code.errors.get(f), Some(&CodeError::UnresolvedIdentifier));
}

#[test]
fn identifier_that_resolves_to_multiple_functions_is_an_error() {
    // If a function shares a name with a different type of function, then an
    // identifier with that name should result in an error.

    let host = Host::from_functions(["identity"]);
    let mut code = Code::default();

    compile_all("identity 1", &host, &mut code);

    let identity = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .next()
        .unwrap();
    assert_eq!(
        code.errors.get(identity),
        Some(&CodeError::MultiResolvedIdentifier)
    );
}

#[test]
fn missing_function_call_argument_is_an_error() {
    // A function call with a missing argument is an error. If an argument is
    // added, that error goes away.

    let host = Host::from_functions(["f"]);
    let mut code = Code::default();

    compile_all("f", &host, &mut code);
    let f = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .next()
        .unwrap();
    assert_eq!(code.errors.get(f), Some(&CodeError::MissingArgument));

    compile_all("1", &host, &mut code);
    let f = code
        .fragments()
        .get(&code.root())
        .body
        .ids()
        .next()
        .unwrap();
    assert_eq!(code.errors.get(f), None);
}
