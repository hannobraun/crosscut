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
    let replacements = Vec::new();
    let mut next_action = compile_token(
        to_replace,
        replacement_token,
        children,
        replacements,
        change_set.nodes,
        change_set.errors,
        packages,
    );

    loop {
        next_action = next_action.perform(change_set, packages);

        if let ReplaceAction::Finish { path } = next_action {
            break path;
        }
    }
}

#[derive(Debug)]
enum ReplaceAction {
    UpdateChildren {
        path: NodePath,
        children: Children,
        replacements: Vec<Replacement>,
    },
    UpdatePath {
        replacement: Replacement,
        parent: Option<NodePath>,
        replacements: Vec<Replacement>,
    },
    Finish {
        path: NodePath,
    },
}

impl ReplaceAction {
    fn perform(self, change_set: &mut NewChangeSet, _: &Packages) -> Self {
        match self {
            Self::UpdateChildren {
                path,
                children,
                replacements,
            } => {
                // comment added to force more readable formatting
                update_children(
                    path,
                    children,
                    replacements,
                    change_set.nodes,
                    change_set.errors,
                )
            }
            Self::UpdatePath {
                replacement,
                parent,
                replacements,
            } => {
                // comment added to force more readable formatting
                update_path(replacement, parent, replacements, change_set)
            }
            action @ Self::Finish { .. } => action,
        }
    }
}

fn compile_token(
    path: NodePath,
    token: String,
    children: Children,
    mut replacements: Vec<Replacement>,
    nodes: &mut Nodes,
    errors: &mut Errors,
    packages: &Packages,
) -> ReplaceAction {
    let token = Token {
        text: &token,
        children,
    };
    let replacement = token.compile(nodes, errors, packages);

    let replacement = Replacement {
        replaced: path,
        replacement,
    };

    if let Some(parent) = replacement.replaced.parent().cloned() {
        let parent_node = nodes.get(parent.hash());

        let mut next_children = parent_node.to_children();
        next_children.replace(&replacement.replaced, replacement.replacement);

        replacements.push(replacement);

        ReplaceAction::UpdateChildren {
            path: parent,
            children: next_children,
            replacements,
        }
    } else {
        ReplaceAction::UpdatePath {
            replacement,
            parent: None,
            replacements,
        }
    }
}

fn update_children(
    path: NodePath,
    children: Children,
    mut replacements: Vec<Replacement>,
    nodes: &mut Nodes,
    errors: &mut Errors,
) -> ReplaceAction {
    let mut expression = nodes.get(path.hash()).clone();

    // TASK: Add comment about panics.
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

    if let Some(parent) = replacement.replaced.parent().cloned() {
        let parent_node = nodes.get(parent.hash());

        let mut next_children = parent_node.to_children();
        next_children.replace(&replacement.replaced, replacement.replacement);

        replacements.push(replacement);

        ReplaceAction::UpdateChildren {
            path: parent,
            children: next_children,
            replacements,
        }
    } else {
        ReplaceAction::UpdatePath {
            replacement,
            parent: None,
            replacements,
        }
    }
}

fn update_path(
    replacement: Replacement,
    parent: Option<NodePath>,
    mut replacements: Vec<Replacement>,
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
            replacements,
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
