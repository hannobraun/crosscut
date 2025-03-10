use std::{collections::BTreeMap, fmt};

use super::{Expression, NodePath};

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
    EmptyNodeWithMultipleChildren,
    UnresolvedIdentifier { candidates: Vec<Expression> },
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyNodeWithMultipleChildren => {
                write!(f, "empty node with multiple children")?;
            }
            Self::UnresolvedIdentifier { candidates } => {
                write!(f, "unresolved syntax node")?;

                if !candidates.is_empty() {
                    write!(f, " (could resolve to")?;

                    for (i, candidate) in candidates.iter().enumerate() {
                        if i == 0 {
                            write!(f, " ")?;
                        } else {
                            write!(f, ", ")?;
                        }

                        match candidate {
                            Expression::HostFunction { .. } => {
                                write!(f, "host function")?;
                            }
                            Expression::IntrinsicFunction { .. } => {
                                write!(f, "intrinsic function")?;
                            }
                        }
                    }

                    write!(f, ")")?;
                }
            }
        }

        Ok(())
    }
}
