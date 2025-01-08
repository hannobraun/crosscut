use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Code {
    pub fragments: Vec<Fragment>,
    pub function_calls: BTreeMap<usize, HostFunction>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fragment {
    Expression { expression: Expression },
    UnexpectedToken { token: Token },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    LiteralValue { value: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
