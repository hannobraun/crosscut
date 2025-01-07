use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Expression>,
    pub function_calls: BTreeMap<usize, HostFunction>,
}

#[derive(Debug)]
pub enum Expression {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct HostFunction {
    pub signature: FunctionType,
}

#[derive(Clone, Copy, Debug)]
pub struct FunctionType {
    pub input: (),
    pub output: (),
}
