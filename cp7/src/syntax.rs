use std::collections::HashMap;

use crate::value::Value;

pub struct Syntax {
    inner: HashMap<SyntaxHandle, SyntaxFragment>,
}

impl Syntax {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn add(&mut self, fragment: SyntaxFragment) -> SyntaxHandle {
        // This is a placeholder. Eventually, we need to add a hash that
        // uniquely addresses the fragment.
        let handle = SyntaxHandle {};
        self.inner.insert(handle, fragment);
        handle
    }

    pub fn get(&self, handle: SyntaxHandle) -> SyntaxFragment {
        // This shouldn't ever panic, as we currently only ever add fragments,
        // never remove them, and only ever create handles for fragments we add.
        self.inner.get(&handle).cloned().unwrap()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SyntaxHandle {}

#[derive(Clone, Debug)]
pub struct SyntaxTree {
    pub elements: Vec<SyntaxFragment>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyntaxFragment {
    pub payload: SyntaxElement,
    pub next: Option<SyntaxHandle>,
}

#[derive(Clone, Debug)]
pub enum SyntaxElement {
    FnRef(String),
    Value(Value),
}
