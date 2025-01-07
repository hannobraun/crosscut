#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    InvalidNumber { invalid: String },
    LiteralNumber { value: f64 },
}
