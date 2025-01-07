use super::{
    code::Code, compiler::compile, host::Host, interpreter::Interpreter,
};

#[test]
fn call_to_host_function() {
    let host = Host::from_function_names(["host_fn"]);
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();

    compile("host_fn 1", &host, &mut code);

    let (id, value) = loop {
        if let Some(call) = interpreter.step(&code) {
            break call;
        }
    };

    assert_eq!(id, host.function_by_name("host_fn").unwrap().id);
    assert_eq!(value, 1.);
}
