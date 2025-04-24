use std::collections::BTreeMap;

use crate::language::packages::FunctionId;

use super::{Node, NodeHash};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Errors {
    inner: BTreeMap<NodeHash<Node>, CodeError>,
}

impl Errors {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get(&self, hash: &NodeHash<Node>) -> Option<&CodeError> {
        self.inner.get(hash)
    }

    pub fn insert(&mut self, hash: NodeHash<Node>, error: CodeError) {
        self.inner.insert(hash, error);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    TooFewChildren,
    TooManyChildren,
    UnresolvedIdentifier {
        candidates: Vec<CandidateForResolution>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateForResolution {
    Literal { literal: Literal },
    ProvidedFunction { id: FunctionId },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Literal {
    Function,
    Integer { value: i32 },
    Tuple,
}
