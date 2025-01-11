use itertools::Itertools;

use crate::language::{
    code::{Code, CodeError, Token},
    compiler::tests::infra::compile,
    host::Host,
};

#[test]
fn code_after_expression_is_an_error() {
    // An expression returns a value. That value can be returned when the
    // program finishes, or it can be used as the argument of a function call.
    //
    // Either way, any code that comes after an expression makes no sense, and
    // is an error.

    let host = Host::empty();

    let mut code = Code::default();
    compile("1 2", &host, &mut code);

    let (a, b) = code
        .fragments()
        .get(&code.root)
        .body
        .ids()
        .collect_tuple()
        .unwrap();

    assert_eq!(code.errors.get(a), None);
    assert_eq!(
        code.errors.get(b),
        Some(&CodeError::Fragment {
            err: crate::language::code::FragmentError::UnexpectedToken {
                token: Token::LiteralNumber { value: 2 }
            }
        }),
    );
}

#[test]
fn unresolved_identifier_is_an_error() {
    // An identifier that does not refer to a function is an error.

    let host = Host::empty();

    let mut code = Code::default();
    compile("f 1", &host, &mut code);

    let f = code.fragments().get(&code.root).body.ids().next().unwrap();

    assert!(code.errors.contains_key(f));
}

#[test]
fn missing_function_call_argument_is_an_error() {
    // A function call with a missing argument is an error. If an argument is
    // added, that error goes away.

    let host = Host::from_functions(["f"]);

    let mut code = Code::default();

    compile("f", &host, &mut code);
    let f = code.fragments().get(&code.root).body.ids().next().unwrap();
    assert!(code.errors.contains_key(f));

    compile("1", &host, &mut code);
    let f = code.fragments().get(&code.root).body.ids().next().unwrap();
    assert!(!code.errors.contains_key(f));
}
