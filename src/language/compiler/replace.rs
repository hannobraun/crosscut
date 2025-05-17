use crate::language::code::{
    NewChangeSet, NodeHash, NodePath, Nodes, SiblingIndex, SyntaxNode,
};

pub fn replace_node_and_update_parents(
    to_replace: NodePath,
    replacement: NodeHash,
    change_set: &mut NewChangeSet,
) -> NodePath {
    let mut replacement = Replacement {
        replaced: to_replace,
        replacement,
    };

    // The replacements that are being made, as we propagate the initial
    // replacement to the root node. We need to remember those, as later, we
    // need to step from the root down again, to update the `NodePath`s of all
    // replaced nodes.
    let mut replacements = Vec::new();

    while let Some(parent) = replacement.replaced.parent().cloned() {
        replacement = update_children(
            parent,
            replacement,
            &mut replacements,
            change_set.nodes,
        );
    }

    let mut next_action = ReplaceAction::UpdatePath {
        replacement,
        parent: None,
    };

    loop {
        next_action = match next_action {
            ReplaceAction::UpdatePath {
                replacement,
                parent,
            } => {
                let path = update_path(replacement, parent, change_set);

                if let Some(rep) = replacements.pop() {
                    let Some(sibling_index) = rep.replaced.sibling_index()
                    else {
                        unreachable!(
                            "The replaced node has a parent, so it must have a \
                             sibling index."
                        );
                    };

                    ReplaceAction::UpdatePath {
                        replacement: rep,
                        parent: Some(path).map(|path| (path, sibling_index)),
                    }
                } else {
                    ReplaceAction::Finish { path }
                }
            }
            ReplaceAction::Finish { path } => {
                break path;
            }
        };
    }
}

#[derive(Debug)]
enum ReplaceAction {
    UpdatePath {
        replacement: Replacement,
        parent: Option<(NodePath, SiblingIndex)>,
    },
    Finish {
        path: NodePath,
    },
}

fn update_children(
    path: NodePath,
    replacement: Replacement,
    replacements: &mut Vec<Replacement>,
    nodes: &mut Nodes,
) -> Replacement {
    let mut expression = nodes.get(path.hash()).clone();

    match &mut expression {
        SyntaxNode::Apply {
            expression,
            argument,
        } => {
            if &expression.hash == replacement.replaced.hash() {
                expression.hash = replacement.replacement;
            } else if argument == replacement.replaced.hash() {
                *argument = replacement.replacement;
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
            if parameter == replacement.replaced.hash() {
                *parameter = replacement.replacement;
            } else if !body.replace(
                &replacement.replaced,
                replacement.replacement,
                1,
            ) {
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
            let was_replaced = children.replace(
                &replacement.replaced,
                replacement.replacement,
                0,
            );

            assert!(
                was_replaced,
                "Tried to replace child that is not present.",
            );
        }
    }

    replacements.push(replacement);

    Replacement {
        replaced: path,
        replacement: nodes.insert(expression),
    }
}

fn update_path(
    replacement: Replacement,
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
