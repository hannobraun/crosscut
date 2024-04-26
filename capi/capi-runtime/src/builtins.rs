#![allow(unused)]

use crate::{data_stack::PopFromEmptyStack, Value};

use super::data_stack::DataStack;

pub fn add(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let Some(c) = a.0.checked_add(b.0) else {
        return Err(Error::IntegerOverflow);
    };

    data_stack.push(Value(c));

    Ok(())
}

pub fn copy(data_stack: &mut DataStack) -> Result {
    let mut i = data_stack.pop()?;

    data_stack.save(i.0);
    let a = data_stack.clone();
    data_stack.restore();

    data_stack.push(a);

    Ok(())
}

pub fn drop(data_stack: &mut DataStack) -> Result {
    let i = data_stack.pop()?;

    data_stack.save(i.0);
    data_stack.pop();
    data_stack.restore();

    Ok(())
}

pub fn mul(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let c = a.0 * b.0;

    data_stack.push(Value(c));

    Ok(())
}

pub fn place(data_stack: &mut DataStack) -> Result {
    let mut i = data_stack.pop()?;
    let mut a = data_stack.pop()?;

    data_stack.save(i.0);
    data_stack.push(a);
    data_stack.restore();

    Ok(())
}

pub fn store(data_stack: &mut DataStack, mem: &mut [u8]) -> Result {
    let value = data_stack.pop()?;
    let addr = data_stack.pop()?;

    let value: u8 = value.0.try_into().unwrap();
    mem[addr.0] = value;

    data_stack.push(addr);

    Ok(())
}

pub fn sub(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    let c = a.0.wrapping_sub(b.0);

    data_stack.push(Value(c));

    Ok(())
}

pub fn swap(data_stack: &mut DataStack) -> Result {
    let b = data_stack.pop()?;
    let a = data_stack.pop()?;

    data_stack.push(b);
    data_stack.push(a);

    Ok(())
}

pub fn take(data_stack: &mut DataStack) -> Result {
    let mut i = data_stack.pop()?;

    data_stack.save(i.0);
    let a = data_stack.pop()?;
    data_stack.restore();

    data_stack.push(a);

    Ok(())
}

pub type Result = std::result::Result<(), Error>;

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    thiserror::Error,
)]
pub enum Error {
    #[error("Integer overflow")]
    IntegerOverflow,

    #[error(transparent)]
    PopFromEmptyStack(#[from] PopFromEmptyStack),
}
