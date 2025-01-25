use crate::lang::{
    code::{Body, Codebase, FragmentKind, Node},
    compiler,
    host::Host,
};

pub fn compile_all(input: &str, host: &Host, code: &mut Codebase) {
    for token in input.split_whitespace() {
        let to_replace = code.append_to(
            &code.find_innermost_fragment_with_valid_body(),
            Node {
                kind: FragmentKind::Empty,
                body: Body::default(),
            },
        );

        compiler::compile_and_replace(token, &to_replace, host, code);
    }
}
