use super::FragmentId;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Body {
    pub inner: Vec<FragmentId>,
}
