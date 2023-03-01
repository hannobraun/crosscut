use super::{
    builtins, Bindings, DataStack, Expression, Expressions, Functions, Value,
};

pub fn evaluate(
    expressions: &Expressions,
    functions: &Functions,
    data_stack: &mut DataStack,
    bindings: &mut Bindings,
) -> Result<(), FunctionNotFound> {
    for expression in expressions {
        evaluate_expression(expression, functions, data_stack, bindings)?;
    }

    Ok(())
}

fn evaluate_expression(
    expression: &Expression,
    functions: &Functions,
    data_stack: &mut DataStack,
    bindings: &mut Bindings,
) -> Result<(), FunctionNotFound> {
    match expression {
        Expression::Block(expressions) => {
            data_stack.push(Value::Block(expressions.clone()));
            Ok(())
        }
        Expression::List(expressions) => {
            let mut list_stack = DataStack::new();
            evaluate(expressions, functions, &mut list_stack, bindings)?;
            let list = Value::List(list_stack.into_iter().collect());
            data_stack.push(list);
            Ok(())
        }
        Expression::Fn(fn_name) => {
            evaluate_fn(fn_name, functions, data_stack, bindings)
        }
        Expression::Name(name) => {
            data_stack.push(Value::Name(name.clone()));
            Ok(())
        }
    }
}

fn evaluate_fn(
    fn_name: &str,
    functions: &Functions,
    data_stack: &mut DataStack,
    bindings: &mut Bindings,
) -> Result<(), FunctionNotFound> {
    if let Some(value) = bindings.resolve(fn_name) {
        data_stack.push(value);
        return Ok(());
    }

    if let Some(builtin) = builtins::get(fn_name) {
        builtin(functions, data_stack, bindings)?;
        return Ok(());
    }

    if let Ok(value) = fn_name.parse::<u8>() {
        data_stack.push(Value::U8(value));
        return Ok(());
    }

    // If we land here, it's not a builtin function.
    let function =
        functions.resolve(fn_name).ok_or_else(|| FunctionNotFound {
            name: fn_name.into(),
        })?;

    evaluate(&function.body, functions, data_stack, bindings)?;

    Ok(())
}

#[derive(Debug)]
pub struct FunctionNotFound {
    pub name: String,
}
