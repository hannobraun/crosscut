use crate::lang::{
    self,
    host::Host,
    interpreter::{StepResult, Value},
};

#[test]
fn identity() {
    // There should be an identity function that returns its argument unchanged,
    // to be used as a placeholder wherever a function is needed, but having any
    // actual behavior is not necessary or desired.

    let host = Host::empty();
    let mut lang = lang::Instance::new();

    lang.edit("identity 1", &host);
    lang.interpreter.reset(&lang.code);

    let step = lang.interpreter.step(&lang.code);
    assert_eq!(step, StepResult::CallToIntrinsicFunction);

    let step = lang.interpreter.step(&lang.code);
    assert_eq!(
        step,
        StepResult::Finished {
            output: Value::Integer { value: 1 }
        }
    );
}
