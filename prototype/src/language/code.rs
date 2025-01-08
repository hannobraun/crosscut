use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Code {
    pub fragments: Vec<Fragment>,
    pub function_calls: BTreeMap<usize, HostFunction>,
}

#[derive(Clone, Debug)]
pub enum Fragment {
    Expression { expression: Expression },
    UnexpectedToken { token: Token },
}

#[derive(Clone, Debug)]
pub enum Expression {
    LiteralValue { value: f64 },
}

#[derive(Clone, Debug)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct HostFunction {
    pub id: usize,
}
