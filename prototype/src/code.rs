#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}
