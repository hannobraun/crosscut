use crate::language::interpreter::InterpreterState;

use super::{
    code::Code, compiler::compile, host::Host, interpreter::Interpreter,
};

#[test]
fn call_to_host_function() {
    // The host can define functions which Crosscut code can call. This should
    // result in the interpreter notifying the host of this call, so it may
    // handle it.

    let host = Host::from_function_names(["host_fn"]);
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("host_fn 1", &host, &mut code);

    let (id, input) = loop {
        if let InterpreterState::CallToHostFunction { id, input } =
            interpreter.step(&code)
        {
            break (id, input);
        }
    };

    assert_eq!(id, host.function_by_name("host_fn").unwrap().id);
    assert_eq!(input, 1.);
}
