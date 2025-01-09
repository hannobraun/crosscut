use crate::language::{code::Code, compiler, host::Host};

pub fn compile(input: &str, code: &mut Code) {
    let mut copy_of_code = code.clone();

    compiler::compile(input, &Host, code);

    // The tests pass the input code in a simple manner. But things should work
    // the same, if it's passed in multiple updates.
    for input in input.split_whitespace() {
        compiler::compile(input, &Host, &mut copy_of_code);
    }
    assert_eq!(*code, copy_of_code);
}
