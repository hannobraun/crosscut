use std::collections::HashMap;

use crate::value::Value;

pub struct Syntax {
    pub inner: HashMap<SyntaxHandle, SyntaxElement>,
}

impl Syntax {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

pub struct SyntaxHandle {}

#[derive(Clone, Debug)]
pub struct SyntaxTree {
    pub elements: Vec<SyntaxElement>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SyntaxElement {
    FnRef(String),
    Value(Value),
}
