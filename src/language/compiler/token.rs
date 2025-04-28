use crate::language::{
    code::{
        CandidateForResolution, Children, CodeError, Errors, Expression,
        Function, Literal, NodeHash, Nodes,
    },
    packages::Packages,
};

pub struct Token<'r> {
    pub text: &'r str,
    pub children: Children,
}

impl Token<'_> {
    pub fn compile(
        self,
        nodes: &mut Nodes,
        errors: &mut Errors,
        packages: &Packages,
    ) -> NodeHash<Expression> {
        let (node, maybe_error) = if self.text.is_empty() {
            (Expression::Empty, None)
        } else if let Some(node) = resolve_keyword(self.text, nodes) {
            (node, None)
        } else {
            match resolve_function(self.text, self.children, packages, nodes) {
                Ok((node, maybe_err)) => (node, maybe_err),
                Err((children, candidates)) => (
                    Expression::Error {
                        node: self.text.to_string(),
                        children,
                    },
                    Some(CodeError::UnresolvedIdentifier { candidates }),
                ),
            }
        };

        let hash = nodes.insert(node);
        if let Some(error) = maybe_error {
            errors.insert(hash, error);
        }

        hash
    }
}

fn resolve_keyword(name: &str, nodes: &mut Nodes) -> Option<Expression> {
    match name {
        "apply" => {
            let [function, argument] = [nodes.insert(Expression::Empty); 2];
            Some(Expression::Apply { function, argument })
        }
        "self" => Some(Expression::Recursion),
        _ => None,
    }
}

fn resolve_function(
    name: &str,
    children: Children,
    packages: &Packages,
    nodes: &mut Nodes,
) -> Result<
    (Expression, Option<CodeError>),
    (Children, Vec<CandidateForResolution>),
> {
    let provided_function = packages.resolve_function(name);
    let literal = resolve_literal(name);

    match (provided_function, literal) {
        (Some(id), None) => Ok((Expression::ProvidedFunction { id }, None)),
        (None, Some(literal)) => match literal {
            Literal::Function => {
                let [parameter, body] = [nodes.insert(Expression::Empty); 2];

                Ok((
                    Expression::Function {
                        function: Function { parameter, body },
                    },
                    None,
                ))
            }
            Literal::Integer { value } => Ok(node_with_no_child_or_error(
                || Expression::Number { value },
                name,
                children,
            )),
            Literal::Tuple => {
                Ok((Expression::Tuple { values: children }, None))
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
    node: impl FnOnce() -> Expression,
    token: &str,
    children: Children,
) -> (Expression, Option<CodeError>) {
    if children.is_empty() {
        (node(), None)
    } else {
        (
            Expression::Error {
                node: token.to_string(),
                children,
            },
            Some(CodeError::TooManyChildren),
        )
    }
}
