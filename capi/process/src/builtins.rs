use crate::{value::IntegerOverflow, Effect, Instruction, Instructions, Stack};

pub fn builtin_by_name(name: &str) -> Option<Builtin> {
    let builtin = match name {
        "add_i8" => add_i8,
        "add_i32" => add_i32,
        "add_u8" => add_u8,
        "add_u8_wrap" => add_u8_wrap,
        "and" => and,
        "brk" => brk,
        "copy" => copy,
        "div_i32" => div_i32,
        "div_u8" => div_u8,
        "drop" => drop,
        "eq" => eq,
        "greater_i8" => greater_i8,
        "greater_i32" => greater_i32,
        "greater_u8" => greater_u8,
        "i32_to_i8" => i32_to_i8,
        "if" => if_,
        "mul_i32" => mul_i32,
        "mul_u8_wrap" => mul_u8_wrap,
        "neg_i32" => neg_i32,
        "not" => not,
        "remainder_i32" => remainder_i32,
        "sub_i32" => sub_i32,
        "sub_u8" => sub_u8,
        "sub_u8_wrap" => sub_u8_wrap,

        _ => {
            return None;
        }
    };

    Some(builtin)
}

pub type Builtin = fn(&mut Stack, &Instructions) -> Result;

fn add_i8(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i8()?;
    let b = b.to_i8()?;

    let Some(c) = a.checked_add(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn add_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    let Some(c) = a.checked_add(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn add_u8(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let Some(c) = a.checked_add(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn add_u8_wrap(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let c = a.wrapping_add(b);
    stack.push_operand(c);

    Ok(())
}

fn and(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let c = if a.0 == [0; 4] || b.0 == [0; 4] { 0 } else { 1 };

    stack.push_operand(c);

    Ok(())
}

fn brk(_: &mut Stack, _: &Instructions) -> Result {
    Err(Effect::Breakpoint)
}

fn copy(stack: &mut Stack, _: &Instructions) -> Result {
    let a = stack.pop_operand()?;

    stack.push_operand(a);
    stack.push_operand(a);

    Ok(())
}

fn div_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    if b == 0 {
        return Err(Effect::DivideByZero);
    }
    let Some(c) = a.checked_div(b) else {
        // Can't be divide by zero. Already handled that.
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn div_u8(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    if b == 0 {
        return Err(Effect::DivideByZero);
    }
    let Some(c) = a.checked_div(b) else {
        // Can't be divide by zero. Already handled that.
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn drop(stack: &mut Stack, _: &Instructions) -> Result {
    stack.pop_operand()?;
    Ok(())
}

fn eq(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let c = if a.0 == b.0 { 1 } else { 0 };

    stack.push_operand(c);

    Ok(())
}

fn greater_i8(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i8()?;
    let b = b.to_i8()?;

    let c = if a > b { 1 } else { 0 };

    stack.push_operand(c);

    Ok(())
}

fn greater_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    let c = if a > b { 1 } else { 0 };

    stack.push_operand(c);

    Ok(())
}

fn greater_u8(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let c = if a > b { 1 } else { 0 };

    stack.push_operand(c);

    Ok(())
}

fn i32_to_i8(stack: &mut Stack, _: &Instructions) -> Result {
    let v = stack.pop_operand()?;

    let v = v.to_i32();
    let v: i8 = v.try_into()?;

    stack.push_operand(v);

    Ok(())
}

/// # Evaluate a condition and evaluate a closure accordingly
///
/// ## Implementation Note
///
/// This duplicates function calling logic that also exists in evaluator. This
/// should be temporary.
///
/// This duplicated logic used to be consolidated within [`Stack::push_frame`],
/// but moved out as part of an effort to move tail call elimination to compile-
/// time.
///
/// As part of the same effort, this function will likely be removed. It should
/// eventually get replaced by something equivalent that doesn't need to
/// duplicate code.
fn if_(stack: &mut Stack, instructions: &Instructions) -> Result {
    let else_ = stack.pop_operand()?;
    let then = stack.pop_operand()?;
    let condition = stack.pop_operand()?;

    let (evaluate, discard) = if condition.0 == [0, 0, 0, 0] {
        (else_, then)
    } else {
        (then, else_)
    };

    let evaluate = evaluate.to_u32();
    let (address, environment) = {
        let mut function = stack.closures.remove(&evaluate).unwrap();

        let branch = function.branches.remove(0);
        assert_eq!(
            function.branches.len(),
            0,
            "`if` does not support pattern-matching functions"
        );

        (branch.start, function.environment)
    };

    let discard = discard.to_u32();
    stack.closures.remove(&discard);

    let mut arguments = Vec::new();
    for (name, value) in environment {
        arguments.push((name, value));
    }

    let is_tail_call = {
        let next_instruction = instructions
            .get(&stack.next_instruction)
            .expect("Expected instruction referenced on stack to exist");

        *next_instruction == Instruction::Return
    };

    if is_tail_call {
        stack.reuse_frame();
    } else {
        stack.push_frame()?;
    }

    stack
        .bindings_mut()
        .expect("Currently executing; stack frame must exist")
        .extend(arguments);

    stack.next_instruction = address;

    Ok(())
}

fn mul_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    let Some(c) = a.checked_mul(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn mul_u8_wrap(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let c = a.wrapping_mul(b);
    stack.push_operand(c);

    Ok(())
}

fn neg_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let a = stack.pop_operand()?;

    let a = a.to_i32();

    if a == i32::MIN {
        return Err(IntegerOverflow.into());
    }
    let b = -a;

    stack.push_operand(b);

    Ok(())
}

fn not(stack: &mut Stack, _: &Instructions) -> Result {
    let a = stack.pop_operand()?;

    let b = if a.0 == [0; 4] { 1 } else { 0 };
    stack.push_operand(b);

    Ok(())
}

fn remainder_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    if b == 0 {
        return Err(Effect::DivideByZero);
    }
    let c = a % b;

    stack.push_operand(c);

    Ok(())
}

fn sub_i32(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_i32();
    let b = b.to_i32();

    let Some(c) = a.checked_sub(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn sub_u8(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let Some(c) = a.checked_sub(b) else {
        return Err(IntegerOverflow.into());
    };

    stack.push_operand(c);

    Ok(())
}

fn sub_u8_wrap(stack: &mut Stack, _: &Instructions) -> Result {
    let b = stack.pop_operand()?;
    let a = stack.pop_operand()?;

    let a = a.to_u8()?;
    let b = b.to_u8()?;

    let c = a.wrapping_sub(b);
    stack.push_operand(c);

    Ok(())
}

pub type Result = std::result::Result<(), Effect>;
