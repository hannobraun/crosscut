use std::collections::BTreeSet;

use super::{Body, Fragment, FragmentId, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: Body,
    pub errors: BTreeSet<FragmentId>,
}

impl Code {
    pub fn entry(&self) -> Option<FragmentId> {
        self.root.inner.first().copied()
    }

    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn fragment_by_id(&self, id: &FragmentId) -> &Fragment {
        self.fragments().get(id)
    }

    pub fn root(&self) -> impl Iterator<Item = &Fragment> {
        self.root.ids().map(|hash| self.fragment_by_id(hash))
    }

    pub fn push(&mut self, fragment: Fragment) -> FragmentId {
        let id = self.fragments.insert(fragment);
        self.root.inner.push(id);
        id
    }

    /// # Indicate whether this is complete, meaning contains an expression
    ///
    /// The presence of errors has no significance for the return value of this
    /// function. Its purpose, rather, is to indicate whether the addition of
    /// more fragments would also result in the addition of _more_ errors.
    ///
    /// ## Implementation Note
    ///
    /// Eventually, this method would move to a type representing something like
    /// a branch, or branch body. That it is defined on `Code` is only a
    /// consequence of the current state of development.
    pub fn is_complete(&self) -> bool {
        self.root()
            .any(|fragment| matches!(fragment, Fragment::Expression { .. }))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: usize, argument: FragmentId },
    LiteralValue { value: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
