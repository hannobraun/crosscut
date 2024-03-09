use super::data::Data;

pub struct Evaluator {
    data: Data,
}

impl Evaluator {
    pub fn new(data: &[u8]) -> Self {
        let data = Data::new(&data);
        Self { data }
    }

    pub fn push_args(
        &mut self,
        args: impl IntoIterator<Item = u8>,
        data: &mut [u8],
    ) {
        for b in args {
            self.data.push(b, data);
        }
    }

    pub fn evaluate(&mut self, code: &[u8], data: &mut [u8]) {
        let mut code_ptr = 0;

        loop {
            let Some(&instruction) = code.get(code_ptr) else {
                break;
            };

            match instruction {
                opcode::TERMINATE => {
                    break;
                }
                opcode::PUSH => {
                    code_ptr += 1;
                    let value = code[code_ptr];

                    self.data.push(value, data);
                }
                opcode::STORE => {
                    let address = self.data.pop(data);
                    let value = self.data.pop(data);

                    self.data.store(address, value, data);
                }
                opcode::CLONE => {
                    let value = self.data.pop(data);
                    self.data.push(value, data);
                    self.data.push(value, data);
                }
                opcode => {
                    let opcode_as_char: char = opcode.into();
                    panic!("Unknown opcode: `{opcode_as_char}` ({opcode:#x})");
                }
            }

            code_ptr += 1;
        }
    }
}

mod opcode {
    pub const TERMINATE: u8 = 0x00;
    pub const PUSH: u8 = 0x01;
    // 0x02 reserved for `load`
    pub const STORE: u8 = 0x03;
    pub const CLONE: u8 = 0x04;
}

#[cfg(test)]
mod tests {
    use super::{opcode, Evaluator};

    // I've started simplifying these tests, so each only depends on the
    // instruction it actually tests, not a whole lot of others for test setup.
    // This went well for the first tests, but failed for the others. The
    // problem is, there's currently no way to set up the stack pointer.
    //
    // Maybe `Evaluator` just needs a way to override the stack pointer then.

    #[test]
    fn terminate() {
        evaluate([opcode::TERMINATE], []);
        // This should not run forever, or cause any kind of panic.
    }

    #[test]
    fn push() {
        let data = evaluate([opcode::PUSH, 255], [0]);
        assert_eq!(data, [255]);
    }

    #[test]
    fn store() {
        let data = evaluate(
            [
                0x01, // push
                255,  // value
                0x01, // push
                0,    // address
                0x03, // store
                0x00, // terminate
            ],
            [0; 2],
        );
        assert_eq!(data[..1], [255]);
    }

    #[test]
    fn clone() {
        let data = evaluate(
            [
                0x01, // push
                255,  // value
                0x04, // clone
                0x00, // terminate
            ],
            [0; 2],
        );
        assert_eq!(data[data.len() - 2..], [255, 255]);
    }

    fn evaluate<const C: usize, const D: usize>(
        code: [u8; C],
        mut data: [u8; D],
    ) -> [u8; D] {
        let mut evaluator = Evaluator::new(&data);

        evaluator.evaluate(&code, &mut data);
        data
    }
}
