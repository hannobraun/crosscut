use crate::language::code::{
    Errors, NewChangeSet, NodeHash, NodePath, Nodes, SiblingIndex, SyntaxNode,
};

pub fn replace_node_and_update_parents(
    to_replace: NodePath,
    replacement: NodeHash,
    change_set: &mut NewChangeSet,
) -> NodePath {
    let replacement = Replacement {
        replaced: to_replace,
        replacement,
    };

    let mut replacements = Vec::new();
    let mut next_action =
        if let Some(parent) = replacement.replaced.parent().cloned() {
            ReplaceAction::UpdateChildren {
                path: parent,
                replacement,
            }
        } else {
            ReplaceAction::UpdatePath {
                replacement,
                parent: None,
            }
        };

    loop {
        next_action = match next_action {
            ReplaceAction::UpdateChildren { path, replacement } => {
                let replacement = update_children(
                    path,
                    replacement,
                    &mut replacements,
                    change_set.nodes,
                    change_set.errors,
                );

                if let Some(parent) = replacement.replaced.parent().cloned() {
                    ReplaceAction::UpdateChildren {
                        path: parent,
                        replacement,
                    }
                } else {
                    ReplaceAction::UpdatePath {
                        replacement,
                        parent: None,
                    }
                }
            }
            ReplaceAction::UpdatePath {
                replacement,
                parent,
            } => {
                // comment added to force more readable formatting
                update_path(replacement, parent, &mut replacements, change_set)
            }
            ReplaceAction::Finish { path } => {
                break path;
            }
        };
    }
}

#[derive(Debug)]
enum ReplaceAction {
    UpdateChildren {
        path: NodePath,
        replacement: Replacement,
    },
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
    errors: &mut Errors,
) -> Replacement {
    let mut expression = nodes.get(path.hash()).clone();

    match &mut expression {
        SyntaxNode::Apply {
            expression: a,
            argument: b,
        }
        | SyntaxNode::Function {
            parameter: a,
            body: b,
        } => {
            if a == replacement.replaced.hash() {
                *a = replacement.replacement;
            } else if b == replacement.replaced.hash() {
                *b = replacement.replacement;
            } else {
                panic!("Expected to replace child, but could not find it.");
            }
        }

        SyntaxNode::AddValue
        | SyntaxNode::Binding { name: _ }
        | SyntaxNode::Empty
        | SyntaxNode::Identifier {
            name: String { .. },
        }
        | SyntaxNode::Number { value: _ }
        | SyntaxNode::ProvidedFunction { name: _ }
        | SyntaxNode::Recursion => {
            panic!("Node has no children. Can't replace one.");
        }

        SyntaxNode::Tuple {
            values: children,
            add_value: NodeHash { .. },
        }
        | SyntaxNode::Test {
            name: String { .. },
            children,
        } => {
            children.replace(&replacement.replaced, replacement.replacement);
        }
    }

    replacements.push(replacement);

    let replacement = Replacement {
        replaced: path,
        replacement: nodes.insert(expression),
    };

    // Updating a child doesn't change anything that could affect an error on
    // the parent. So we need to preserve that.
    if let Some(error) = errors.get(replacement.replaced.hash()) {
        errors.insert(replacement.replacement, error.clone());
    }

    replacement
}

fn update_path(
    replacement: Replacement,
    parent: Option<(NodePath, SiblingIndex)>,
    replacements: &mut Vec<Replacement>,
    change_set: &mut NewChangeSet,
) -> ReplaceAction {
    let path = NodePath::new(replacement.replacement, parent, change_set.nodes);

    change_set.replace(&replacement.replaced, &path);

    if let Some(replacement) = replacements.pop() {
        let Some(sibling_index) = replacement.replaced.sibling_index() else {
            unreachable!(
                "The replaced node has a parent, so it must have a sibling \
                index."
            );
        };

        ReplaceAction::UpdatePath {
            replacement,
            parent: Some(path).map(|path| (path, sibling_index)),
        }
    } else {
        ReplaceAction::Finish { path }
    }
}

#[derive(Clone, Debug)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash,
}
