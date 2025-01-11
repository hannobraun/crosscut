use std::{collections::BTreeMap, fmt};

use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};

use super::{Body, Expression, Token};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Fragments {
    inner: BTreeMap<FragmentId, Fragment>,
}

impl Fragments {
    pub fn get(&self, id: &FragmentId) -> &Fragment {
        let Some(fragment) = self.inner.get(id) else {
            panic!(
                "Fragment with ID `{id:?}` not found. This should never \
                happen, unless you are mixing and matching data structures \
                from different instances of `Code`."
            );
        };
        fragment
    }

    pub fn insert(&mut self, fragment: Fragment) -> FragmentId {
        let id = FragmentId::generate(&fragment);

        self.inner.insert(id, fragment);

        id
    }
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, udigest::Digestable)]
pub struct FragmentId {
    hash: [u8; 32],
}
impl FragmentId {
    fn generate(fragment: &Fragment) -> Self {
        let hash = udigest::hash::<blake3::Hasher>(fragment).into();
        Self { hash }
    }
}

impl fmt::Debug for FragmentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_STANDARD_NO_PAD.encode(self.hash))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub struct Fragment {
    pub kind: FragmentKind,
    pub body: Body,
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum FragmentKind {
    Expression { expression: Expression },
    UnexpectedToken { token: Token },
}
