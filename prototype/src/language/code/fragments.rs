use std::{collections::BTreeMap, fmt};

use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};

use super::{Body, Expression};

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

/// # The ID of a fragment
///
/// Fragment IDs are based on hashing. This means that different fragments
/// should result in different hashes. Hash collisions, meaning the same IDs for
/// equal hashes, should be exceedingly unlikely.
///
/// Another consequence of this, is that equal fragments end up with the same
/// ID, even if they are located in different parts of the syntax tree. This is
/// not a problem, because if those fragments are truly equal, there's really no
/// reason to not also consider them identical.
///
/// There is one aspect here that might be a bit unintuitive: That fragments
/// that are rendered similarly in the editor, can actually still be distinct.
/// For example:
///
/// - Two calls to the same function `f` can still be distinct fragments, as
///   their arguments are included in the fragment, and thus influence their ID.
/// - A simple value like `1` can be a perfectly valid expression fragment,
///   while the same value in another place would be an unexpected token. The
///   compiler would emit those as different kinds of fragments, which would
///   then have different IDs.
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

impl Fragment {
    /// # Returns the body of the fragment, if this kind can have a valid one
    ///
    /// Fragments that can have a valid body, are all fragments that allow
    /// nesting. That includes function calls, for example, whose argument is
    /// nested within the function call fragment's body.
    pub fn valid_body(&self) -> Option<&Body> {
        match self.kind {
            FragmentKind::Root
            | FragmentKind::Expression {
                expression: Expression::FunctionCall { .. },
            } => Some(&self.body),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum FragmentKind {
    Root,
    Expression { expression: Expression },
    Error { err: FragmentError },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum FragmentError {
    IntegerOverflow { value: String },
    UnresolvedIdentifier { name: String },
}
