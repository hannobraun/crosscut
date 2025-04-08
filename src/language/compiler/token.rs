use std::cmp::Ordering;

use itertools::Itertools;

use crate::language::{
    code::{
        CandidateForResolution, Children, CodeError, Errors, Literal,
        NewChangeSet, Node, NodeHash, NodePath,
    },
    packages::Packages,
};

pub struct Token<'r> {
    pub text: &'r str,
    pub parent: Option<&'r NodePath>,
    pub sibling_index: usize,
    pub children: Children,
}

impl Token<'_> {
    pub fn compile(
        self,
        change_set: &mut NewChangeSet,
        _: &mut Errors,
        packages: &Packages,
    ) -> (NodeHash, Option<CodeError>) {
        compile_token(self, change_set, packages)
    }
}

fn compile_token(
    token: Token,
    change_set: &mut NewChangeSet,
    packages: &Packages,
) -> (NodeHash, Option<CodeError>) {
    // We're about to need that, to correctly compile function parameters.
    let _ = token.parent;
    let _ = token.sibling_index;

    let (node, maybe_error) = if token.text.is_empty() {
        node_with_one_child_or_error(
            |child| Node::Empty { child },
            token.text,
            token.children,
        )
    } else if let Some((node, maybe_err)) =
        resolve_keyword(token.text, &token.children)
    {
        (node, maybe_err)
    } else {
        match resolve_function(token.text, token.children, packages) {
            Ok((node, maybe_err)) => (node, maybe_err),
            Err((children, candidates)) => (
                Node::Error {
                    node: token.text.to_string(),
                    children,
                },
                Some(CodeError::UnresolvedIdentifier { candidates }),
            ),
        }
    };

    let hash = change_set.add(node);

    (hash, maybe_error)
}

fn resolve_keyword(
    name: &str,
    children: &Children,
) -> Option<(Node, Option<CodeError>)> {
    match name {
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
            Literal::Integer { value } => {
                if children.is_empty() {
                    Ok((Node::LiteralNumber { value }, None))
                } else {
                    Ok((
                        Node::Error {
                            node: name.to_string(),
                            children,
                        },
                        Some(CodeError::TooManyChildren),
                    ))
                }
            }
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
