use crate::language::{
    code::{NewChangeSet, NodeHash, NodePath, Nodes, SiblingIndex, SyntaxNode},
    compiler::Tuple,
};

use super::{Apply, Function};

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

    while let Some((parent, sibling_index)) = to_replace.parent() {
        let parent = parent.clone();

        let next_replacement = update_children(
            &parent,
            &to_replace,
            replacement,
            sibling_index,
            change_set.nodes,
        );

        replacements.push(Replacement {
            replaced: to_replace,
            replacement,
            sibling_index,
        });

        to_replace = parent;
        replacement = next_replacement;
    }

    let mut path = update_path(&to_replace, replacement, None, change_set);

    while let Some(replacement) = replacements.pop() {
        let parent = Some(path).map(|path| (path, replacement.sibling_index));
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
    sibling_index: SiblingIndex,
    nodes: &mut Nodes,
) -> NodeHash {
    let mut expression = nodes.get(path.hash()).clone();

    let node = match &mut expression {
        SyntaxNode::Apply {
            expression,
            argument,
        } => {
            let mut apply = Apply {
                expression: *expression,
                argument: *argument,
            };

            let replaced_expression = apply.expression_mut().replace(
                to_replace.hash(),
                &sibling_index,
                replacement,
            );
            let replaced_argument = apply.argument_mut().replace(
                to_replace.hash(),
                &sibling_index,
                replacement,
            );

            if !replaced_expression && !replaced_argument {
                panic!("Expected to replace child, but could not find it.");
            }

            apply.into_syntax_node()
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
            let mut function = Function {
                parameter: *parameter,
                body: body.clone(),
            };

            let replaced_parameter = function.parameter_mut().replace(
                to_replace.hash(),
                &sibling_index,
                replacement,
            );

            if replaced_parameter {
            } else if !function.body_mut().replace(
                to_replace.hash(),
                &sibling_index,
                replacement,
            ) {
                panic!("Expected to replace child, but could not find it.");
            }

            function.into_syntax_node()
        }

        SyntaxNode::Tuple {
            values: children,
            add_value,
        } => {
            let mut tuple = Tuple {
                values: children.clone(),
                add_value: *add_value,
            };

            let was_replaced = tuple.values_mut().replace(
                to_replace.hash(),
                &sibling_index,
                replacement,
            );
            assert!(
                was_replaced,
                "Tried to replace child that is not present.",
            );

            tuple.into_syntax_node()
        }
    };

    nodes.insert(node)
}

fn update_path(
    replaced: &NodePath,
    replacement: NodeHash,
    parent: Option<(NodePath, SiblingIndex)>,
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
    sibling_index: SiblingIndex,
}
