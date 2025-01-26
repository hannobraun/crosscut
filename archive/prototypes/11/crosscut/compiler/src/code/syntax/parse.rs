use std::result;

use crate::code::{
    tokens::{Keyword::*, NoMoreTokens, Punctuator::*, Token, Tokens},
    Index, IndexMap, Signature,
};

use super::{
    repr::types::SyntaxType, Binding, Branch, BranchLocation, Comment,
    Expression, Function, FunctionLocation, Member, MemberLocation,
    NamedFunction, Parameter,
};

/// # Parse the provided tokens
///
/// ## Implementation Note
///
/// This compiler pass currently panics when it encounters an unexpected token.
/// It would be better, if it encoded the error into its output instead. This is
/// non-trivial though, compared to other compiler passes that do that.
///
/// For example, if an identifier can not be resolved, this only affects that
/// identifier, and it is quite easy to encode that by having a [`Expression`]
/// variant for unresolved identifiers.
///
/// In the case of parsing, however, an unexpected token would likely result in
/// the parser not knowing what to do with the following tokens, even it can be
/// taught to recover eventually. Those tokens would also have to be encoded
/// into the code representation. Otherwise, viewing that representation (which
/// would include the error) makes no sense to a user.
///
/// I'm inclined to leave this be for now. Panicking certainly works well enough
/// in the current setup. Once we have a code database, it will no longer do.
/// But then we also need a completely different parser architecture.
///
/// It's probably not worth solving this non-trivial problem for the current
/// architecture, for little gain, only to re-solve it again for the new
/// architecture, once that is necessary.
pub fn parse(mut tokens: Tokens) -> IndexMap<NamedFunction> {
    let mut named_functions = IndexMap::default();

    loop {
        let index = named_functions.next_index();

        let function = match parse_named_function(&mut tokens, index) {
            Ok(function) => function,
            Err(Error::NoMoreTokens(NoMoreTokens)) => {
                break;
            }
            Err(err) => {
                panic!("Parser error: {err:?}");
            }
        };

        let actual_index = named_functions.push(function);
        assert_eq!(
            index, actual_index,
            "Function has a different index than was initially assumed.",
        );
    }

    named_functions
}

fn parse_named_function(
    tokens: &mut Tokens,
    index: Index<NamedFunction>,
) -> Result<NamedFunction> {
    let comment = parse_comment(tokens)?;
    let name = parse_function_name(tokens)?;

    let location = FunctionLocation::Named { index };
    let function = parse_function(tokens, location)?;

    Ok(NamedFunction {
        comment,
        name,
        inner: function,
    })
}

fn parse_comment(tokens: &mut Tokens) -> Result<Option<Comment>> {
    let mut lines = Vec::new();

    let mut indentation = None;

    while let Token::CommentLine { line } = tokens.peek()? {
        let offset = indentation.or_else(|| {
            let first_after_whitespace = line.split_whitespace().next()?;
            let whitespace = line.split(first_after_whitespace).next()?;
            let indentation = whitespace.chars().count();
            Some(indentation)
        });
        indentation = offset;

        let offset = offset.unwrap_or(0);

        let line = line.chars().skip(offset);
        lines.push(line.collect());

        tokens.take()?;
    }

    let comment = if lines.is_empty() {
        None
    } else {
        Some(Comment { lines })
    };

    Ok(comment)
}

fn parse_function_name(tokens: &mut Tokens) -> Result<String> {
    let name = match tokens.take()? {
        Token::Identifier { name } => name,
        token => {
            return Err(Error::UnexpectedToken { actual: token });
        }
    };

    match tokens.take()? {
        Token::Punctuator(Introducer) => {}
        token => {
            return Err(Error::UnexpectedToken { actual: token });
        }
    }

    Ok(name)
}

fn parse_function(
    tokens: &mut Tokens,
    location: FunctionLocation,
) -> Result<Function> {
    let mut branches = IndexMap::default();

    match tokens.take()? {
        Token::Keyword(Fn) => {}
        token => {
            return Err(Error::UnexpectedToken { actual: token });
        }
    }

    loop {
        let location = BranchLocation {
            parent: Box::new(location.clone()),
            index: branches.next_index(),
        };

        let Some(branch) = parse_branch(tokens, location)? else {
            break;
        };

        branches.push(branch);
    }

    match tokens.take()? {
        Token::Keyword(End) => {}
        token => {
            return Err(Error::UnexpectedToken { actual: token });
        }
    }

    Ok(Function { branches })
}

fn parse_branch(
    tokens: &mut Tokens,
    location: BranchLocation,
) -> Result<Option<Branch>> {
    let comment = parse_comment(tokens)?;

    match tokens.peek()? {
        Token::Keyword(Br) => {
            tokens.take()?;
        }
        Token::Keyword(End) => {
            // This would be the `end` of the function, not the branch, and
            // hence our sign that no more branches can be parsed here.
            return Ok(None);
        }
        _ => {
            let token = tokens.take()?;
            return Err(Error::UnexpectedToken { actual: token });
        }
    }

    let parameters = parse_branch_parameters(tokens)?;
    let body = parse_branch_body(tokens, location)?;

    Ok(Some(Branch {
        comment,
        parameters,
        body,
    }))
}

fn parse_branch_parameters(tokens: &mut Tokens) -> Result<IndexMap<Parameter>> {
    let mut parameters = IndexMap::default();

    loop {
        if let Token::Punctuator(Transformer) = tokens.peek()? {
            tokens.take()?;
            break;
        }

        let parameter = parse_parameter(tokens)?;
        parameters.push(parameter);

        match tokens.take()? {
            Token::Punctuator(Delimiter) => {
                // If we have a delimiter, then we're good here. Next loop
                // iteration, we'll either parse the next parameter, or if it
                // was the last one, find the start of the branch body.
                continue;
            }
            Token::Punctuator(Transformer) => {
                // The last parameter doesn't need a delimiter, so this is fine
                // too.
                break;
            }
            token => {
                return Err(Error::UnexpectedToken { actual: token });
            }
        }
    }

    Ok(parameters)
}

fn parse_parameter(tokens: &mut Tokens) -> Result<Parameter> {
    let parameter = match tokens.take()? {
        Token::Identifier { name } => {
            let type_ = parse_type_annotation(tokens)?;

            Parameter::Binding {
                binding: Binding { name },
                type_,
            }
        }
        Token::IntegerLiteral { value } => Parameter::Literal {
            value: value.into(),
        },
        token => {
            return Err(Error::UnexpectedToken { actual: token });
        }
    };

    Ok(parameter)
}

fn parse_branch_body(
    tokens: &mut Tokens,
    location: BranchLocation,
) -> Result<IndexMap<Member>> {
    let mut body = IndexMap::default();

    loop {
        if let Token::Keyword(End) = tokens.peek()? {
            tokens.take()?;
            break;
        }

        let location = MemberLocation {
            parent: Box::new(location.clone()),
            index: body.next_index(),
        };
        let member = parse_member(tokens, location)?;
        body.push(member);
    }

    Ok(body)
}

fn parse_member(
    tokens: &mut Tokens,
    location: MemberLocation,
) -> Result<Member> {
    let member = if let Some(comment) = parse_comment(tokens)? {
        Member::Comment(comment)
    } else {
        let (expression, signature) = parse_expression(tokens, location)?;

        Member::Expression {
            expression,
            signature,
        }
    };

    Ok(member)
}

fn parse_expression(
    tokens: &mut Tokens,
    location: MemberLocation,
) -> Result<(Expression, Option<Signature<SyntaxType>>)> {
    let expression = if let Token::Keyword(Fn) = tokens.peek()? {
        let location = FunctionLocation::Local { location };
        parse_function(tokens, location)
            .map(|function| Expression::LocalFunction { function })?
    } else {
        match tokens.take()? {
            Token::Identifier { name } => Expression::Identifier { name },
            Token::IntegerLiteral { value } => Expression::LiteralNumber {
                value: value.into(),
            },
            token => {
                return Err(Error::UnexpectedToken { actual: token });
            }
        }
    };

    let signature = parse_signature_annotation(tokens)?;

    Ok((expression, signature))
}

fn parse_type_annotation(tokens: &mut Tokens) -> Result<Option<SyntaxType>> {
    let Token::Punctuator(Introducer) = tokens.peek()? else {
        return Ok(None);
    };
    tokens.take()?;

    let type_ = parse_type(tokens)?;

    Ok(Some(type_))
}

fn parse_signature_annotation(
    tokens: &mut Tokens,
) -> Result<Option<Signature<SyntaxType>>> {
    let Token::Punctuator(Introducer) = tokens.peek()? else {
        return Ok(None);
    };
    tokens.take()?;

    let terminator = Token::Punctuator(Terminator);
    let signature = parse_signature(tokens, terminator)?;

    Ok(Some(signature))
}

fn parse_signature(
    tokens: &mut Tokens,
    terminator: Token,
) -> Result<Signature<SyntaxType>> {
    let mut inputs = Vec::new();

    loop {
        if let Token::Punctuator(Transformer) = tokens.peek()? {
            tokens.take()?;
            break;
        }

        let type_ = parse_type(tokens)?;
        inputs.push(type_);

        match tokens.take()? {
            Token::Punctuator(Delimiter) => {
                continue;
            }
            Token::Punctuator(Transformer) => {
                break;
            }
            token => {
                return Err(Error::UnexpectedToken { actual: token });
            }
        }
    }

    let mut outputs = Vec::new();

    loop {
        // We already handle the terminator at the end of the loop. We have to
        // also handle it here, since it's allowed to not have any outputs,
        // which would make the terminator the first token we encounter in this
        // loop.
        if *tokens.peek()? == terminator {
            tokens.take()?;
            break;
        }

        let type_ = parse_type(tokens)?;
        outputs.push(type_);

        match tokens.take()? {
            Token::Punctuator(Delimiter) => {
                continue;
            }
            token if token == terminator => {
                break;
            }
            token => {
                return Err(Error::UnexpectedToken { actual: token });
            }
        }
    }

    Ok(Signature { inputs, outputs })
}

fn parse_type(tokens: &mut Tokens) -> Result<SyntaxType> {
    let type_ = match tokens.take()? {
        Token::Identifier { name } => SyntaxType::Identifier { name },
        Token::Keyword(Fn) => {
            let terminator = Token::Keyword(End);
            let signature = parse_signature(tokens, terminator)?;
            SyntaxType::Function { signature }
        }
        token => {
            return Err(Error::UnexpectedToken { actual: token });
        }
    };

    Ok(type_)
}

type Result<T> = result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    NoMoreTokens(#[from] NoMoreTokens),

    #[error("Unexpected token: {actual:?}")]
    UnexpectedToken { actual: Token },
}
