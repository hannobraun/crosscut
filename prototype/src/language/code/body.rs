use super::{Expression, Fragment, FragmentId, FragmentKind, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq, udigest::Digestable)]
pub struct Body {
    inner: Vec<FragmentId>,
}

impl Body {
    pub fn push_fragment(
        &mut self,
        fragment: Fragment,
        fragments: &mut Fragments,
    ) -> FragmentId {
        let id = fragments.insert(fragment);
        self.push_id(id);
        id
    }

    pub fn push_id(&mut self, id: FragmentId) {
        self.inner.push(id);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn entry(&self) -> Option<&FragmentId> {
        self.inner.first()
    }

    pub fn ids(&self) -> impl DoubleEndedIterator<Item = &FragmentId> {
        self.inner.iter()
    }

    pub fn fragments<'r>(
        &'r self,
        fragments: &'r Fragments,
    ) -> impl Iterator<Item = &'r Fragment> {
        self.ids().map(|hash| fragments.get(hash))
    }

    pub fn expression<'r>(
        &'r self,
        fragments: &'r Fragments,
    ) -> impl Iterator<Item = (&'r Expression, &'r Body)> {
        self.fragments(fragments).filter_map(|fragment| {
            if let FragmentKind::Expression { expression } = &fragment.kind {
                Some((expression, &fragment.body))
            } else {
                None
            }
        })
    }

    pub fn replace(
        &mut self,
        to_replace: &FragmentId,
        replace_with: Fragment,
        fragments: &mut Fragments,
    ) -> FragmentId {
        for id in self.inner.iter_mut() {
            if id == to_replace {
                let id_of_replacement = fragments.insert(replace_with);
                *id = id_of_replacement;
                return id_of_replacement;
            }
        }

        panic!(
            "Expecting `Body::replace` to replace a fragment, but none was \
            found."
        );
    }
}
