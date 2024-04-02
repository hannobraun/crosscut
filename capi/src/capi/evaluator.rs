use super::{
    builtins, code::Code, compiler::Instruction, data_stack::DataStack,
};

#[derive(Debug)]
pub struct Evaluator {
    pub code: Code,
    pub call_stack: Vec<usize>,
    pub data_stack: DataStack,
}

impl Evaluator {
    pub fn new(code: Code) -> Self {
        Self {
            code,
            call_stack: Vec::new(),
            data_stack: DataStack::new(),
        }
    }

    pub fn evaluate(&mut self, entry: usize, frame: &mut [u8]) {
        let mut current_instruction = entry;

        loop {
            let instruction = self.code.instructions[current_instruction];
            current_instruction += 1;

            match instruction {
                Instruction::CallBuiltin { name } => match name {
                    "add" => builtins::add(&mut self.data_stack),
                    "clone2" => builtins::clone2(&mut self.data_stack),
                    "mul" => builtins::mul(&mut self.data_stack),
                    "store" => builtins::store(&mut self.data_stack, frame),
                    _ => panic!("Unknown builtin: `{name}`"),
                },
                Instruction::CallFunction { name } => {
                    let address = self.code.symbols.resolve(name);
                    self.call_stack.push(current_instruction);
                    current_instruction = address;
                }
                Instruction::PushValue(value) => self.data_stack.push(value),
                Instruction::Return => {
                    let Some(return_address) = self.call_stack.pop() else {
                        break;
                    };

                    current_instruction = return_address;
                }
            }
        }
    }
}
