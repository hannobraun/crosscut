use itertools::Itertools;

use crate::language::{
    code::Code, compiler::tests::infra::compile, host::Host,
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

    let (_a, b) = code.root.ids().collect_tuple().unwrap();
    assert!(code.errors.contains(b));
}
