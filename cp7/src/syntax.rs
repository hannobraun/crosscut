use std::{collections::HashMap, fmt};

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
        let hash = {
            let mut hasher = blake3::Hasher::new();

            hasher.update(fragment.payload.to_string().as_bytes());
            if let Some(next) = fragment.next {
                hasher.update(next.hash.as_bytes());
            }

            hasher.finalize()
        };

        let handle = SyntaxHandle { hash };
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
pub struct SyntaxHandle {
    hash: blake3::Hash,
}

#[derive(Clone, Debug)]
pub struct SyntaxFragment {
    pub payload: SyntaxElement,
    pub next: Option<SyntaxHandle>,
}

#[derive(Clone, Debug)]
pub enum SyntaxElement {
    Value(Value),
    Word(String),
}

impl fmt::Display for SyntaxElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxElement::Value(value) => write!(f, "{value}"),
            SyntaxElement::Word(word) => write!(f, "{word}"),
        }
    }
}
