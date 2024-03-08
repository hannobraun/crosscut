use std::iter;

use super::data::Data;

pub struct Evaluator {
    code: Vec<u8>,
    data: Vec<u8>,
}

impl Evaluator {
    pub fn new(program: &[u8]) -> Self {
        // I want to know when I go beyond certain thresholds, just out of
        // interest. Keeping the limits as low as possible here, to make sure I
        // notice.
        const CODE_SIZE: usize = 8;
        const DATA_SIZE: usize = 2;

        let mut code: Vec<_> = iter::repeat(0).take(CODE_SIZE).collect();
        let data = iter::repeat(0).take(DATA_SIZE).collect();

        code[..program.len()].copy_from_slice(&program);

        Self { code, data }
    }

    pub fn evaluate(
        &mut self,
        arguments: impl IntoIterator<Item = u8>,
    ) -> &[u8] {
        let mut code_ptr = 0;
        let mut stack = Data::new(&mut self.data);

        for b in arguments {
            stack.push(b);
        }

        loop {
            let instruction = self.code[code_ptr];

            match instruction {
                // `terminate` - Terminate the program
                b't' => {
                    break;
                }

                // `push` - Push a value to the stack
                b'p' => {
                    code_ptr += 1;
                    let value = self.code[code_ptr];

                    stack.push(value);
                }

                // `store` - Store data in memory
                b'S' => {
                    let address = stack.pop();
                    let value = stack.pop();

                    stack.store(address, value);
                }

                opcode => {
                    let opcode_as_char: char = opcode.into();
                    panic!("Unknown opcode: `{opcode_as_char}` ({opcode:#x})");
                }
            }

            code_ptr += 1;
        }

        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluator;

    #[test]
    fn terminate() {
        let mut evaluator = Evaluator::new(&[b't']);

        evaluator.evaluate([]);
        // This should not run forever, or cause any kind of panic.
    }

    #[test]
    fn push() {
        let mut evaluator = Evaluator::new(&[b'p', 255, b't']);
        let data = evaluator.evaluate([]);
        assert_eq!(data[data.len() - 1..], [255]);
    }

    #[test]
    fn store() {
        let mut evaluator = Evaluator::new(&[b'p', 255, b'p', 0, b'S', b't']);
        let data = evaluator.evaluate([]);
        assert_eq!(data[0..1], [255]);
    }
}
