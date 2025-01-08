use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Code {
    pub fragments: Vec<Fragment>,
    pub function_calls: BTreeMap<usize, HostFunction>,
}

#[derive(Debug)]
pub enum Fragment {
    UnexpectedToken { token: Token },
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
