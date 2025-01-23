use crate::lang::{
    self,
    code::Code,
    host::Host,
    interpreter::{Interpreter, StepResult, Value},
};

#[test]
fn call_to_host_function() {
    // The host can define functions. Those functions take one argument, return
    // one value, and can be called from Crosscut code.

    let output = compile_and_run("half 64");
    assert_eq!(output, Value::Integer { value: 32 });
}

#[test]
fn nested_calls_to_host_function() {
    // It is possible use a function call as the argument of another function
    // call.

    let output = compile_and_run("half half 64");
    assert_eq!(output, Value::Integer { value: 16 });
}

fn compile_and_run(input_code: &str) -> Value {
    let host = Host::from_functions(["half"]);
    let mut lang = lang::Instance::new();

    lang.on_input(input_code, &host);
    run(&lang.code, &mut lang.interpreter)
}

fn run(code: &Code, interpreter: &mut Interpreter) -> Value {
    loop {
        match interpreter.step(code) {
            StepResult::CallToHostFunction { id, input, output } => {
                assert_eq!(id, 0);

                let Value::Integer { value: input } = input;
                let Value::Integer { value: output } = output;

                *output = input / 2;
            }
            StepResult::Finished { output } => {
                break output;
            }
            state => {
                panic!("Unexpected state: {state:#?}");
            }
        }
    }
}
