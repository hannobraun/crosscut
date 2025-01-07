#[derive(Debug, Default)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    Identifier { name: String },
    LiteralNumber { value: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct FunctionType {
    pub input: (),
    pub output: (),
}
