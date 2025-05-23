use std::cmp::Ordering;

use itertools::Itertools;

use crate::language::{
    code::{
        CandidateForResolution, Children, CodeError, Errors, Literal,
        NewChangeSet, Node, NodeHash, NodePath, SiblingIndex,
    },
    packages::Packages,
};

pub struct Token<'r> {
    pub text: &'r str,
    pub parent: Option<&'r NodePath>,
    pub sibling_index: SiblingIndex,
    pub children: Children,
}

impl Token<'_> {
    pub fn compile(
        self,
        change_set: &mut NewChangeSet,
        errors: &mut Errors,
        packages: &Packages,
    ) -> NodeHash {
        // We're about to need that, to correctly compile function parameters.
        let _ = self.parent;
        let _ = self.sibling_index;

        let (node, maybe_error) = if self.text.is_empty() {
            node_with_no_child_or_error(
                || Node::Empty,
                self.text,
                self.children,
            )
        } else if let Some((node, maybe_err)) =
            resolve_keyword(self.text, &self.children)
        {
            (node, maybe_err)
        } else {
            match resolve_function(self.text, self.children, packages) {
                Ok((node, maybe_err)) => (node, maybe_err),
                Err((children, candidates)) => (
                    Node::Error {
                        node: self.text.to_string(),
                        children,
                    },
                    Some(CodeError::UnresolvedIdentifier { candidates }),
                ),
            }
        };

        let hash = change_set.add(node);
        if let Some(error) = maybe_error {
            errors.insert(hash, error);
        }

        hash
    }
}

fn resolve_keyword(
    name: &str,
    children: &Children,
) -> Option<(Node, Option<CodeError>)> {
    match name {
        "apply" => {
            let (node, maybe_error) = if children.inner.len() == 2 {
                let Some([function, argument]) =
                    children.iter().copied().collect_array()
                else {
                    unreachable!(
                        "Just checked that there are exactly two children."
                    );
                };

                (Node::Application { function, argument }, None)
            } else {
                (
                    Node::Error {
                        node: name.to_string(),
                        children: children.clone(),
                    },
                    // We should be setting an error here, but there's no test
                    // to cover this yet.
                    //
                    // It might be best to wait and see how the ongoing cleanup
                    // of `Node` shakes out, so there hasn't been a huge rush to
                    // get this done.
                    None,
                )
            };

            Some((node, maybe_error))
        }
        "self" => Some(node_with_one_child_or_error(
            |argument| Node::Recursion { argument },
            name,
            children.clone(),
        )),
        _ => None,
    }
}

fn resolve_function(
    name: &str,
    children: Children,
    packages: &Packages,
) -> Result<(Node, Option<CodeError>), (Children, Vec<CandidateForResolution>)>
{
    let provided_function = packages.resolve_function(name);
    let literal = resolve_literal(name);

    match (provided_function, literal) {
        (Some(id), None) => Ok(node_with_one_child_or_error(
            |argument| Node::ProvidedFunction { id, argument },
            name,
            children,
        )),
        (None, Some(literal)) => match literal {
            Literal::Function => {
                if let Some([parameter, body]) =
                    children.iter().copied().collect_array()
                {
                    Ok((Node::LiteralFunction { parameter, body }, None))
                } else {
                    let expected_num = 2;
                    let num_children = children.inner.len();

                    let error = match num_children.cmp(&expected_num) {
                        Ordering::Less => CodeError::TooFewChildren,
                        Ordering::Greater => CodeError::TooManyChildren,
                        Ordering::Equal => {
                            unreachable!(
                                "We already handled the case, of the function \
                                literal having the expected number of children."
                            );
                        }
                    };

                    Ok((
                        Node::Error {
                            node: name.to_string(),
                            children: children.clone(),
                        },
                        Some(error),
                    ))
                }
            }
            Literal::Integer { value } => Ok(node_with_no_child_or_error(
                || Node::LiteralNumber { value },
                name,
                children,
            )),
            Literal::Tuple => {
                Ok((Node::LiteralTuple { values: children }, None))
            }
        },
        (None, None) => {
            let candidates = Vec::new();
            Err((children, candidates))
        }
        (provided_function, literal) => {
            let mut candidates = Vec::new();

            if let Some(id) = provided_function {
                candidates
                    .push(CandidateForResolution::ProvidedFunction { id });
            }
            if let Some(literal) = literal {
                candidates.push(CandidateForResolution::Literal { literal });
            }

            Err((children, candidates))
        }
    }
}

fn resolve_literal(name: &str) -> Option<Literal> {
    if let Ok(value) = name.parse() {
        Some(Literal::Integer { value })
    } else {
        match name {
            "fn" => Some(Literal::Function),
            "tuple" => Some(Literal::Tuple),
            _ => None,
        }
    }
}

fn node_with_no_child_or_error(
    node: impl FnOnce() -> Node,
    token: &str,
    children: Children,
) -> (Node, Option<CodeError>) {
    if children.is_empty() {
        (node(), None)
    } else {
        (
            Node::Error {
                node: token.to_string(),
                children,
            },
            Some(CodeError::TooManyChildren),
        )
    }
}

fn node_with_one_child_or_error(
    node_from_child: impl FnOnce(Option<NodeHash>) -> Node,
    token: &str,
    children: Children,
) -> (Node, Option<CodeError>) {
    if children.is_multiple_children().is_none() {
        let maybe_child = children.is_single_child().copied();
        (node_from_child(maybe_child), None)
    } else {
        (
            Node::Error {
                node: token.to_string(),
                children,
            },
            Some(CodeError::TooManyChildren),
        )
    }
}
