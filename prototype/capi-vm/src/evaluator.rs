use crate::{
    opcode,
    width::{Width, W16, W32, W64, W8},
};

use super::data::Data;

pub struct Evaluator {
    data: Data,
}

impl Evaluator {
    pub fn new(data: &[u8]) -> Self {
        let data = Data::new(data);
        Self { data }
    }

    pub fn push_u8(&mut self, value: u8, data: &mut [u8]) {
        self.data.push([value], data);
    }

    pub fn push_u32(&mut self, value: u32, data: &mut [u8]) {
        self.data.push(value.to_le_bytes(), data);
    }

    pub fn evaluate(&mut self, code: &[u8], data: &mut [u8]) {
        let mut code_ptr = 0;

        loop {
            let Some(&instruction) = code.get(code_ptr) else {
                break;
            };

            let opcode = instruction & 0x3f;
            let width = match instruction >> 6 {
                W8::ENCODING => W8::INFO,
                W16::ENCODING => W16::INFO,
                W32::ENCODING => W32::INFO,
                W64::ENCODING => W64::INFO,
                _ => unreachable!("2 bits can only encode 4 values"),
            };

            match opcode {
                opcode::TERMINATE => {
                    break;
                }
                opcode::PUSH => {
                    code_ptr += 1;
                    let value = code[code_ptr];

                    self.data.push([value], data);
                }
                opcode::DROP => {
                    if width == W8::INFO {
                        self.data.pop::<1>(data);
                    }
                    if width == W16::INFO {
                        self.data.pop::<2>(data);
                    }
                    if width == W32::INFO {
                        self.data.pop::<4>(data);
                    }
                    if width == W64::INFO {
                        self.data.pop::<8>(data);
                    }
                }
                opcode::STORE => {
                    let [address] = self.data.pop(data);
                    let [value] = self.data.pop(data);

                    self.data.store(address, value, data);
                }
                opcode::CLONE => {
                    let [value] = self.data.pop(data);
                    self.data.push([value], data);
                    self.data.push([value], data);
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

#[cfg(test)]
mod tests {
    use crate::{
        opcode,
        width::{Width, W16, W32, W64, W8},
    };

    use super::Evaluator;

    #[test]
    fn terminate() {
        evaluate([opcode::TERMINATE], [], []);
        // This should not run forever, nor cause any kind of panic.
    }

    #[test]
    fn push() {
        let data = evaluate([opcode::PUSH, 255], [0], []);
        assert_eq!(data, [255]);
    }

    #[test]
    fn drop8() {
        let data = evaluate(
            [opcode::DROP | W8::INFO.flag, opcode::PUSH, 255],
            [0],
            [127],
        );
        assert_eq!(data, [255]);
    }
    #[test]
    fn drop16() {
        let data = evaluate(
            [opcode::DROP | W16::INFO.flag, opcode::PUSH, 255],
            [0, 0],
            [127, 127],
        );
        assert_eq!(data, [127, 255]);
    }

    #[test]
    fn drop32() {
        let data = evaluate(
            [opcode::DROP | W32::INFO.flag, opcode::PUSH, 255],
            [0, 0, 0, 0],
            [127, 127, 127, 127],
        );
        assert_eq!(data, [127, 127, 127, 255]);
    }

    #[test]
    fn drop64() {
        let data = evaluate(
            [opcode::DROP | W64::INFO.flag, opcode::PUSH, 255],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [127, 127, 127, 127, 127, 127, 127, 127],
        );
        assert_eq!(data, [127, 127, 127, 127, 127, 127, 127, 255]);
    }

    #[test]
    fn store() {
        let data = evaluate([opcode::STORE], [0, 0, 0], [255, 0]);
        assert_eq!(data, [255, 0, 255]);
    }

    #[test]
    fn clone() {
        let data = evaluate([opcode::CLONE], [0, 0], [255]);
        assert_eq!(data, [255, 255]);
    }

    fn evaluate<const C: usize, const D: usize, const A: usize>(
        code: [u8; C],
        mut data: [u8; D],
        args: [u8; A],
    ) -> [u8; D] {
        let mut evaluator = Evaluator::new(&data);

        for arg in args {
            evaluator.push_u8(arg, &mut data);
        }

        evaluator.evaluate(&code, &mut data);
        data
    }
}
