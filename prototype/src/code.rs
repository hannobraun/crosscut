#[derive(Debug)]
pub struct Code {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    Color { color: [f64; 4] },
}
