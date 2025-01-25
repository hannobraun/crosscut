use super::{Expression, FragmentId, FragmentKind, Node, Nodes};

#[derive(Clone, Debug, Default, Eq, PartialEq, udigest::Digestable)]
pub struct Body {
    children: Vec<FragmentId>,
}

impl Body {
    pub fn push_node(&mut self, node: Node, nodes: &mut Nodes) -> FragmentId {
        let id = nodes.insert(node);
        self.push_id(id);
        id
    }

    pub fn push_id(&mut self, id: FragmentId) {
        self.children.push(id);
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn entry(&self) -> Option<&FragmentId> {
        self.children.first()
    }

    pub fn ids(&self) -> impl DoubleEndedIterator<Item = &FragmentId> {
        self.children.iter()
    }

    pub fn children<'r>(
        &'r self,
        fragments: &'r Nodes,
    ) -> impl Iterator<Item = &'r Node> {
        self.ids().map(|hash| fragments.get(hash))
    }

    pub fn expressions<'r>(
        &'r self,
        fragments: &'r Nodes,
    ) -> impl Iterator<Item = (&'r Expression, &'r Body)> {
        self.children(fragments).filter_map(|fragment| {
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
        replace_with: Node,
        fragments: &mut Nodes,
    ) -> FragmentId {
        for id in self.children.iter_mut() {
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
