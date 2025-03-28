use std::collections::BTreeMap;

use crate::language::packages::FunctionId;

use super::NodePath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Errors {
    inner: BTreeMap<NodePath, CodeError>,
}

impl Errors {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn get(&self, path: &NodePath) -> Option<&CodeError> {
        self.inner.get(path)
    }

    pub fn insert(&mut self, path: NodePath, error: CodeError) {
        self.inner.insert(path, error);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeError {
    OnlyUpToOneChildAllowedForThisNode,
    IntegerLiteralWithChildren,
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
