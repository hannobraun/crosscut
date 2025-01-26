use crate::code::{Index, Type};

use super::types::{InferredType, InferredTypes, Result};

#[derive(Debug)]
pub struct MaybeLocalStack {
    inner: Option<LocalStack>,
}

impl MaybeLocalStack {
    pub fn get(&self) -> Option<&LocalStack> {
        self.inner.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut LocalStack> {
        self.inner.as_mut()
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
    }
}

impl Default for MaybeLocalStack {
    fn default() -> Self {
        Self {
            inner: Some(LocalStack::default()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LocalStack {
    pub inner: Vec<Index<InferredType>>,
}

impl LocalStack {
    pub fn make_direct(
        &self,
        types: &mut InferredTypes,
    ) -> Result<Option<Vec<Type>>> {
        self.inner
            .iter()
            .map(|index| types.resolve(index)?.into_type(types))
            .collect()
    }
}
