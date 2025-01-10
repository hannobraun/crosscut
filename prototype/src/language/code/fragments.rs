use std::collections::BTreeMap;

use super::{Expression, Token};

pub type Fragments = BTreeMap<FragmentId, Fragment>;

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

#[derive(Clone, Debug, PartialEq, udigest::Digestable)]
pub enum Fragment {
    Expression { expression: Expression },
    MissingArgument,
    UnexpectedToken { token: Token },
}
