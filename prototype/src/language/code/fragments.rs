use std::collections::BTreeMap;

use super::{Expression, Token};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Fragments {
    inner: BTreeMap<FragmentId, Fragment>,
}

impl Fragments {
    pub fn get(&self, id: &FragmentId) -> Option<&Fragment> {
        self.inner.get(id)
    }

    pub fn insert(&mut self, fragment: Fragment) -> FragmentId {
        let id = FragmentId::generate(&fragment);

        let existing = self.inner.insert(id, fragment);
        assert!(
            existing.is_none(),
            "Hash collision! This is either incredibly unlikely (as in, less \
            likely than being hit by lightning, according to my understanding) \
            or a bug.",
        );

        id
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable,
)]
pub struct FragmentId {
    hash: [u8; 32],
}
impl FragmentId {
    pub fn generate(fragment: &Fragment) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(fragment).into();
        Self { hash }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Fragment {
    Expression { expression: Expression },
    MissingArgument,
    UnexpectedToken { token: Token },
}
