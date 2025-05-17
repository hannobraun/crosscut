use crate::language::code::{
    NewChangeSet, NodeHash, NodePath, Nodes, SiblingIndex, SyntaxNode,
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

    while let Some((parent, _)) = to_replace.parent_and_sibling_index() {
        let parent = parent.clone();

        let next_replacement = update_children(
            &parent,
            &to_replace,
            replacement,
            change_set.nodes,
        );

        replacements.push(Replacement {
            replaced: to_replace,
            replacement,
        });

        to_replace = parent;
        replacement = next_replacement;
    }

    let mut current_replacement = Replacement {
        replaced: to_replace,
        replacement,
    };
    let mut path = update_path(&current_replacement, None, change_set);

    while let Some(replacement) = replacements.pop() {
        let Some(sibling_index) = replacement.replaced.sibling_index() else {
            unreachable!(
                "The replaced node has a parent, so it must have a sibling \
                index."
            );
        };

        current_replacement = replacement;
        let parent = Some(path).map(|path| (path, sibling_index));
        path = update_path(&current_replacement, parent, change_set);
    }

    path
}

fn update_children(
    path: &NodePath,
    to_replace: &NodePath,
    replacement: NodeHash,
    nodes: &mut Nodes,
) -> NodeHash {
    let mut expression = nodes.get(path.hash()).clone();

    match &mut expression {
        SyntaxNode::Apply {
            expression,
            argument,
        } => {
            if &expression.hash == to_replace.hash() {
                expression.hash = replacement;
            } else if argument == to_replace.hash() {
                *argument = replacement;
            } else {
                panic!("Expected to replace child, but could not find it.");
            }
        }

        SyntaxNode::AddNode
        | SyntaxNode::Binding { name: _ }
        | SyntaxNode::Empty
        | SyntaxNode::Identifier {
            name: String { .. },
        }
        | SyntaxNode::Number { value: _ }
        | SyntaxNode::Recursion => {
            panic!("Node has no children. Can't replace one.");
        }

        SyntaxNode::Function { parameter, body } => {
            if parameter == to_replace.hash() {
                *parameter = replacement;
            } else if !body.replace(to_replace, replacement, 1) {
                panic!("Expected to replace child, but could not find it.");
            }
        }

        SyntaxNode::Tuple {
            values: children,
            add_value: NodeHash { .. },
        }
        | SyntaxNode::Test {
            name: String { .. },
            children,
        } => {
            let was_replaced = children.replace(to_replace, replacement, 0);

            assert!(
                was_replaced,
                "Tried to replace child that is not present.",
            );
        }
    }

    nodes.insert(expression)
}

fn update_path(
    replacement: &Replacement,
    parent: Option<(NodePath, SiblingIndex)>,
    change_set: &mut NewChangeSet,
) -> NodePath {
    let path = NodePath::new(replacement.replacement, parent, change_set.nodes);

    change_set.replace(&replacement.replaced, &path);

    path
}

#[derive(Clone, Debug)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}
