use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Token>,
    pub function_calls: BTreeMap<usize, HostFunction>,
}

#[derive(Debug)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct HostFunction {
    pub id: usize,
}
