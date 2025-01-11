use crate::language::{
    code::{
        Body, Code, Expression, Fragment, FragmentError, FragmentKind,
        FragmentPath, Token,
    },
    host::Host,
};

pub fn compile(input: &str, host: &Host, code: &mut Code) {
    for token in tokenize(input) {
        let append_to = code.find_innermost_fragment_with_valid_body();

        let kind = match parse_token(token, &append_to, code, host) {
            Ok(expression) => FragmentKind::Expression { expression },
            Err(err) => FragmentKind::Error { err },
        };
        let fragment = Fragment {
            kind,
            body: Body::default(),
        };

        let is_error = check_for_error(&fragment).is_some();

        let id = code.append(fragment, append_to);

        if is_error {
            code.errors.insert(id);
        }
    }
}

fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    input
        .split_whitespace()
        .map(|token| match token.parse::<u32>() {
            Ok(value) => Token::LiteralNumber { value },
            Err(_) => Token::Identifier {
                name: token.to_string(),
            },
        })
}

fn parse_token(
    token: Token,
    append_to: &FragmentPath,
    code: &Code,
    host: &Host,
) -> Result<Expression, FragmentError> {
    match token {
        Token::Identifier { name } => {
            if let Some(id) = host.functions_by_name.get(&name).copied() {
                Ok(Expression::FunctionCall { target: id })
            } else {
                Err(FragmentError::UnresolvedIdentifier { name })
            }
        }
        Token::LiteralNumber { value } => {
            let can_append_expression = code
                .fragments()
                .get(append_to.id())
                .body
                .expression(code.fragments())
                .is_none();

            if can_append_expression {
                Ok(Expression::LiteralValue { value })
            } else {
                Err(FragmentError::UnexpectedToken {
                    token: Token::LiteralNumber { value },
                })
            }
        }
    }
}

fn check_for_error(fragment: &Fragment) -> Option<()> {
    match fragment.kind {
        FragmentKind::Expression {
            expression: Expression::FunctionCall { .. },
        } => {
            if fragment.body.is_empty() {
                return Some(());
            }
        }
        FragmentKind::Error { .. } => {
            return Some(());
        }
        _ => {}
    }

    None
}
