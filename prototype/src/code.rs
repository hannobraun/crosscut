#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    LiteralNumber { value: f64 },
    InvalidNumber { invalid: String },
}
