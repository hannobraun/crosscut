use alloc::vec::Vec;

use crate::{
    function::Pattern, Effect, Function, Heap, Instruction, InstructionAddress,
    Instructions, Stack,
};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct Evaluator {
    pub stack: Stack,
    pub next_instruction: InstructionAddress,
}

impl Evaluator {
    pub fn active_instructions(
        &self,
    ) -> impl Iterator<Item = InstructionAddress> + '_ {
        self.stack
            .return_addresses()
            .map(|instruction| {
                // All return addresses point to the _next_ instruction to
                // execute when returning from the frame. We want to return the
                // _current_ active instructions though, so we have to correct
                // that here.
                instruction.previous()
            })
            // This method is usually called when an effect has been triggered,
            // and when that happens, the evaluator does not advance to the next
            // instruction. Therefore, the next instruction does not require the
            // same correction as the return addresses do above.
            .chain([self.next_instruction])
    }

    pub fn step(
        &mut self,
        instructions: Instructions,
        heap: &mut Heap,
    ) -> Result<(), Effect> {
        if self.stack.no_frames_left() {
            return Ok(());
        }

        let current_instruction = instructions
            .get(&self.next_instruction)
            .expect("Expected instruction referenced on stack to exist");
        let next_instruction = self.next_instruction.next();

        self.next_instruction = evaluate_instruction(
            current_instruction,
            next_instruction,
            heap,
            &mut self.stack,
        )?;

        Ok(())
    }
}

fn evaluate_instruction(
    current_instruction: &Instruction,
    next_instruction: InstructionAddress,
    heap: &mut Heap,
    stack: &mut Stack,
) -> Result<InstructionAddress, Effect> {
    match current_instruction {
        Instruction::AddS8 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i8()?;
            let b = b.to_i8()?;

            let Some(c) = a.checked_add(b) else {
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::AddS32 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i32();
            let b = b.to_i32();

            let Some(c) = a.checked_add(b) else {
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::AddU8 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            let Some(c) = a.checked_add(b) else {
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::AddU8Wrap => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            let c = a.wrapping_add(b);
            stack.push_operand(c);
        }
        Instruction::Bind { name } => {
            let value = stack.pop_operand()?;
            stack.define_binding(name.clone(), value);
        }
        Instruction::BindingEvaluate { name } => {
            let Some(bindings) = stack.bindings() else {
                unreachable!(
                    "Can't access bindings, but we're currently executing. An \
                    active stack frame, and therefore bindings, must exist."
                );
            };
            let Some(value) = bindings.get(name).copied() else {
                unreachable!(
                    "Can't find binding `{name}`, but instruction that \
                    evaluates bindings should only be generated for bindings \
                    that exist.\n\
                    \n\
                    Current stack:\n\
                    {:#?}",
                    stack,
                );
            };
            stack.push_operand(value);
        }
        Instruction::CallFunction {
            callee: function,
            is_tail_call,
        } => {
            for branch in &function.branches {
                let mut used_operands = Vec::new();
                let mut argument_operands = Vec::new();
                let mut bound_arguments = Vec::new();

                let mut member_matches = true;
                for parameter in branch.parameters.iter().rev() {
                    let operand = stack.pop_operand()?;
                    used_operands.push(operand);

                    match parameter {
                        Pattern::Identifier { name } => {
                            bound_arguments.push((name.clone(), operand));
                            argument_operands.push(operand);
                        }
                        Pattern::Literal { value } => {
                            member_matches &= *value == operand;
                        }
                    }
                }

                if member_matches {
                    for value in argument_operands.into_iter().rev() {
                        stack.push_operand(value);
                    }

                    if *is_tail_call {
                        stack.reuse_frame();
                    } else {
                        stack.push_frame(next_instruction)?;
                    }

                    return Ok(branch.start);
                } else {
                    for value in used_operands.into_iter().rev() {
                        stack.push_operand(value);
                    }
                }
            }

            return Err(Effect::NoMatch);
        }
        Instruction::ConvertS32ToS8 => {
            let v = stack.pop_operand()?;

            let v = v.to_i32();
            let v: i8 = v.try_into()?;

            stack.push_operand(v);
        }
        Instruction::Copy => {
            let offset_from_top = stack.pop_operand()?.to_usize();

            let Some(value) =
                stack.operands().rev().nth(offset_from_top).copied()
            else {
                return Err(Effect::InvalidArgument);
            };

            stack.push_operand(value);
        }
        Instruction::DivS32 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i32();
            let b = b.to_i32();

            if b == 0 {
                return Err(Effect::DivideByZero);
            }
            let Some(c) = a.checked_div(b) else {
                // Can't be divide by zero. Already handled that.
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::DivU8 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            if b == 0 {
                return Err(Effect::DivideByZero);
            }
            let Some(c) = a.checked_div(b) else {
                // Can't be divide by zero. Already handled that.
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::Drop => {
            stack.pop_operand()?;
        }
        Instruction::Eq => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let c = if a.0 == b.0 { 1 } else { 0 };

            stack.push_operand(c);
        }
        Instruction::Eval { is_tail_call } => {
            // This duplicates code from other places, which is unfortunate,
            // but works for now.
            //
            // See implementation note on `Instruction::Eval` for context on
            // this.

            let function = {
                let index = stack.pop_operand()?;
                let index = index.to_u32();

                heap.closures
                    .remove(&index)
                    .ok_or(Effect::InvalidFunction)?
            };

            for branch in &function.branches {
                let mut used_operands = Vec::new();
                let mut argument_operands = Vec::new();
                let mut bound_arguments = Vec::new();

                let mut member_matches = true;
                for parameter in branch.parameters.iter().rev() {
                    let operand = stack.pop_operand()?;
                    used_operands.push(operand);

                    match parameter {
                        Pattern::Identifier { name } => {
                            bound_arguments.push((name.clone(), operand));
                            argument_operands.push(operand);
                        }
                        Pattern::Literal { value } => {
                            member_matches &= *value == operand;
                        }
                    }
                }

                if member_matches {
                    for value in argument_operands.into_iter().rev() {
                        stack.push_operand(value);
                    }

                    if *is_tail_call {
                        stack.reuse_frame();
                    } else {
                        stack.push_frame(next_instruction)?;
                    }

                    stack
                        .bindings_mut()
                        .expect("Currently executing; stack frame must exist")
                        .extend(function.environment);

                    return Ok(branch.start);
                } else {
                    for value in used_operands.into_iter().rev() {
                        stack.push_operand(value);
                    }
                }
            }

            return Err(Effect::NoMatch);
        }
        Instruction::GreaterS8 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i8()?;
            let b = b.to_i8()?;

            let c = if a > b { 1 } else { 0 };

            stack.push_operand(c);
        }
        Instruction::GreaterS32 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i32();
            let b = b.to_i32();

            let c = if a > b { 1 } else { 0 };

            stack.push_operand(c);
        }
        Instruction::GreaterU8 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            let c = if a > b { 1 } else { 0 };

            stack.push_operand(c);
        }
        Instruction::LogicalAnd => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let c = if a.0 == [0; 4] || b.0 == [0; 4] { 0 } else { 1 };

            stack.push_operand(c);
        }
        Instruction::LogicalNot => {
            let a = stack.pop_operand()?;

            let b = if a.0 == [0; 4] { 1 } else { 0 };
            stack.push_operand(b);
        }
        Instruction::MakeAnonymousFunction {
            branches,
            environment,
        } => {
            let Some(bindings) = stack.bindings() else {
                unreachable!(
                    "We're currently executing. A stack frame, and thus \
                    bindings, must exist."
                );
            };

            let environment = environment
                .iter()
                .cloned()
                .map(|name| {
                    let Some(value) = bindings.get(&name).cloned() else {
                        unreachable!(
                            "Binding `{name}`, from the environment of an \
                            anonymous function, does not exist.\n\
                            \n\
                            A binding that is part of a function's \
                            environment, must exist in the parent scope of \
                            that function.",
                        );
                    };

                    (name, value)
                })
                .collect();

            let index = {
                let index = heap.next_closure;
                heap.next_closure += 1;
                index
            };
            heap.closures.insert(
                index,
                Function {
                    branches: branches.clone(),
                    environment,
                },
            );

            stack.push_operand(index);
        }
        Instruction::MulS32 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i32();
            let b = b.to_i32();

            let Some(c) = a.checked_mul(b) else {
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::MulU8Wrap => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            let c = a.wrapping_mul(b);
            stack.push_operand(c);
        }
        Instruction::NegS32 => {
            let a = stack.pop_operand()?;

            let a = a.to_i32();

            if a == i32::MIN {
                return Err(Effect::IntegerOverflow);
            }
            let b = -a;

            stack.push_operand(b);
        }
        Instruction::Nop => {
            // "no operation"
        }
        Instruction::Push { value } => {
            stack.push_operand(*value);
        }
        Instruction::RemainderS32 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i32();
            let b = b.to_i32();

            if b == 0 {
                return Err(Effect::DivideByZero);
            }
            let c = a % b;

            stack.push_operand(c);
        }
        Instruction::Return => {
            if let Some(return_address) = stack.pop_frame() {
                return Ok(return_address);
            }
        }
        Instruction::SubS32 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_i32();
            let b = b.to_i32();

            let Some(c) = a.checked_sub(b) else {
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::SubU8 => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            let Some(c) = a.checked_sub(b) else {
                return Err(Effect::IntegerOverflow);
            };

            stack.push_operand(c);
        }
        Instruction::SubU8Wrap => {
            let b = stack.pop_operand()?;
            let a = stack.pop_operand()?;

            let a = a.to_u8()?;
            let b = b.to_u8()?;

            let c = a.wrapping_sub(b);
            stack.push_operand(c);
        }
        Instruction::TriggerEffect { effect } => {
            return Err(*effect);
        }
    }

    Ok(next_instruction)
}
