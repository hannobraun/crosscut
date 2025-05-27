use crate::language::code::{
    ChildIndex, NewChangeSet, NodeHash, NodePath, Nodes,
};

pub fn replace_node_and_update_parents(
    to_replace: NodePath,
    replacement: NodeHash,
    change_set: &mut NewChangeSet,
) -> NodePath {
    let mut to_replace = to_replace;
    let mut replacement = replacement;

    // The replacements that are being made, as we propagate the initial
    // replacement to the root node. We need to remember those, as later, we
    // need to step from the root down again, to update the `NodePath`s of all
    // replaced nodes.
    let mut replacements = Vec::new();

    while let Some((parent, index)) = to_replace.parent() {
        let parent = parent.clone();

        let next_replacement = update_children(
            &parent,
            &to_replace,
            replacement,
            index,
            change_set.nodes,
        );

        replacements.push(Replacement {
            replaced: to_replace,
            replacement,
            index,
        });

        to_replace = parent;
        replacement = next_replacement;
    }

    let mut path = update_path(&to_replace, replacement, None, change_set);

    while let Some(replacement) = replacements.pop() {
        let parent = Some(path).map(|path| (path, replacement.index));
        path = update_path(
            &replacement.replaced,
            replacement.replacement,
            parent,
            change_set,
        );
    }

    path
}

fn update_children(
    path: &NodePath,
    to_replace: &NodePath,
    replacement: NodeHash,
    index: ChildIndex,
    nodes: &mut Nodes,
) -> NodeHash {
    let mut node = nodes.get(path.hash()).clone();

    if !node
        .children_mut()
        .replace(to_replace.hash(), &index, replacement)
    {
        panic!("Expected to replace child, but could not find it.");
    }

    nodes.insert(node)
}

fn update_path(
    replaced: &NodePath,
    replacement: NodeHash,
    parent: Option<(NodePath, ChildIndex)>,
    change_set: &mut NewChangeSet,
) -> NodePath {
    let path = NodePath::new(replacement, parent, change_set.nodes);

    change_set.replace(replaced, &path);

    path
}

#[derive(Clone, Debug)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
    index: ChildIndex,
}
