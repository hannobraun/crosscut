use crate::language::{code::Code, compiler, host::Host};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in input.split_whitespace() {
        compiler::compile(token, host, code);
    }
}
