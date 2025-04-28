use std::collections::BTreeMap;

use crate::language::packages::FunctionId;

use super::{Expression, NodeHash};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Errors {
    inner: BTreeMap<NodeHash<Expression>, CodeError>,
}

impl Errors {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get(&self, hash: &NodeHash<Expression>) -> Option<&CodeError> {
        self.inner.get(hash)
    }

    pub fn insert(&mut self, hash: NodeHash<Expression>, error: CodeError) {
        self.inner.insert(hash, error);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
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
