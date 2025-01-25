use super::{Expression, FragmentKind, Node, NodeId, Nodes};

#[derive(Clone, Debug, Default, Eq, PartialEq, udigest::Digestable)]
pub struct Body {
    children: Vec<NodeId>,
}

impl Body {
    pub fn push_node(&mut self, node: Node, nodes: &mut Nodes) -> NodeId {
        let id = nodes.insert(node);
        self.push_id(id);
        id
    }

    pub fn push_id(&mut self, id: NodeId) {
        self.children.push(id);
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn entry(&self) -> Option<&NodeId> {
        self.children.first()
    }

    pub fn ids(&self) -> impl DoubleEndedIterator<Item = &NodeId> {
        self.children.iter()
    }

    pub fn children<'r>(
        &'r self,
        nodes: &'r Nodes,
    ) -> impl Iterator<Item = &'r Node> {
        self.ids().map(|hash| nodes.get(hash))
    }

    pub fn expressions<'r>(
        &'r self,
        nodes: &'r Nodes,
    ) -> impl Iterator<Item = (&'r Expression, &'r Body)> {
        self.children(nodes).filter_map(|fragment| {
            if let FragmentKind::Expression { expression } = &fragment.kind {
                Some((expression, &fragment.body))
            } else {
                None
            }
        })
    }

    pub fn replace(
        &mut self,
        to_replace: &NodeId,
        replace_with: Node,
        nodes: &mut Nodes,
    ) -> NodeId {
        for id in self.children.iter_mut() {
            if id == to_replace {
                let id_of_replacement = nodes.insert(replace_with);
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
