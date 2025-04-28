use crate::language::{
    code::{
        Children, Errors, Expression, Function, NewChangeSet, NodeHash,
        NodePath, Nodes,
    },
    packages::Packages,
};

use super::token::Token;

pub fn replace_node_and_update_parents(
    to_replace: NodePath,
    replacement_token: String,
    children: Children,
    change_set: &mut NewChangeSet,
    packages: &Packages,
) -> NodePath {
    let mut replacements = Vec::new();
    let replacement = compile_token(
        to_replace,
        replacement_token,
        children,
        &mut replacements,
        change_set.nodes,
        change_set.errors,
        packages,
    );

    let mut next_action = if let Some(parent) =
        replacement.replaced.parent().cloned()
    {
        let parent_node = change_set.nodes.get(parent.hash());

        let mut next_children = parent_node.to_children();
        next_children.replace(&replacement.replaced, replacement.replacement);

        replacements.push(replacement);

        ReplaceAction::UpdateChildren {
            path: parent,
            children: next_children,
        }
    } else {
        ReplaceAction::UpdatePath {
            replacement,
            parent: None,
        }
    };

    loop {
        next_action = match next_action {
            ReplaceAction::UpdateChildren { path, children } => {
                let replacement = update_children(
                    path,
                    children,
                    change_set.nodes,
                    change_set.errors,
                );

                if let Some(parent) = replacement.replaced.parent().cloned() {
                    let parent_node = change_set.nodes.get(parent.hash());

                    let mut next_children = parent_node.to_children();
                    next_children.replace(
                        &replacement.replaced,
                        replacement.replacement,
                    );

                    replacements.push(replacement);

                    ReplaceAction::UpdateChildren {
                        path: parent,
                        children: next_children,
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
            action @ ReplaceAction::Finish { .. } => action,
        };

        if let ReplaceAction::Finish { path } = next_action {
            break path;
        }
    }
}

fn compile_token(
    path: NodePath,
    token: String,
    children: Children,
    _: &mut Vec<Replacement>,
    nodes: &mut Nodes,
    errors: &mut Errors,
    packages: &Packages,
) -> Replacement {
    let token = Token {
        text: &token,
        children,
    };
    let replacement = token.compile(nodes, errors, packages);

    Replacement {
        replaced: path,
        replacement,
    }
}

#[derive(Debug)]
enum ReplaceAction {
    UpdateChildren {
        path: NodePath,
        children: Children,
    },
    UpdatePath {
        replacement: Replacement,
        parent: Option<NodePath>,
    },
    Finish {
        path: NodePath,
    },
}

fn update_children(
    path: NodePath,
    children: Children,
    nodes: &mut Nodes,
    errors: &mut Errors,
) -> Replacement {
    let mut expression = nodes.get(path.hash()).clone();

    match &mut expression {
        Expression::Apply {
            function: a,
            argument: b,
        }
        | Expression::Function {
            function:
                Function {
                    parameter: a,
                    body: b,
                },
        } => {
            let [new_a, new_b] = children.expect();

            *a = new_a;
            *b = new_b;
        }

        Expression::Empty
        | Expression::Number { value: _ }
        | Expression::ProvidedFunction { id: _ }
        | Expression::Recursion => {
            let [] = children.expect();
        }

        Expression::Tuple { values } => {
            *values = children;
        }

        Expression::Error {
            node: _,
            children: c,
        } => {
            *c = children;
        }
    }

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
    parent: Option<NodePath>,
    replacements: &mut Vec<Replacement>,
    change_set: &mut NewChangeSet,
) -> ReplaceAction {
    let path = NodePath::new(
        replacement.replacement,
        parent,
        replacement.replaced.sibling_index(),
        change_set.nodes,
    );

    change_set.replace(&replacement.replaced, &path);

    if let Some(replacement) = replacements.pop() {
        ReplaceAction::UpdatePath {
            replacement,
            parent: Some(path),
        }
    } else {
        ReplaceAction::Finish { path }
    }
}

#[derive(Clone, Debug)]
struct Replacement {
    replaced: NodePath,
    replacement: NodeHash<Expression>,
}
