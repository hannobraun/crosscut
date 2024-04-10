#![allow(unused)]

use super::data_stack::DataStack;

pub fn add(data_stack: &mut DataStack) {
    let b = data_stack.pop();
    let a = data_stack.pop();

    let c = a + b;

    data_stack.push(c);
}

pub fn copy(data_stack: &mut DataStack) {
    let mut i = data_stack.pop();

    let mut tmp = Vec::new();
    for _ in 0..i {
        tmp.push(data_stack.pop());
    }

    let a = data_stack.pop();
    data_stack.push(a);

    while let Some(x) = tmp.pop() {
        data_stack.push(x);
    }

    data_stack.push(a);
}

pub fn drop2(data_stack: &mut DataStack) {
    data_stack.pop();
    data_stack.pop();
}

pub fn mul(data_stack: &mut DataStack) {
    let b = data_stack.pop();
    let a = data_stack.pop();

    let c = a * b;

    data_stack.push(c);
}

pub fn store(data_stack: &mut DataStack, mem: &mut [u8]) {
    let value = data_stack.pop();
    let addr = data_stack.pop();

    let value: u8 = value.try_into().unwrap();
    mem[addr] = value;

    data_stack.push(addr);
}

pub fn sub(data_stack: &mut DataStack) {
    let b = data_stack.pop();
    let a = data_stack.pop();

    let c = a.wrapping_sub(b);

    data_stack.push(c);
}

pub fn swap(data_stack: &mut DataStack) {
    let b = data_stack.pop();
    let a = data_stack.pop();

    data_stack.push(b);
    data_stack.push(a);
}
