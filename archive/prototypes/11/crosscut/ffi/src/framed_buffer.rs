use std::collections::VecDeque;

pub struct FramedBuffer<const SIZE: usize> {
    buffer: [u8; SIZE],
    frames: VecDeque<BufferFrame>,
}

impl<const SIZE: usize> FramedBuffer<SIZE> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; SIZE],
            frames: VecDeque::new(),
        }
    }

    pub fn write_frame(&mut self, len: usize) -> &mut [u8] {
        let next_free =
            self.frames.back().copied().unwrap_or_default().ends_before;

        let new_frame = BufferFrame {
            starts_at: next_free,
            ends_before: next_free + len,
        };
        self.frames.push_back(new_frame);

        &mut self.buffer[new_frame.starts_at..new_frame.ends_before]
    }

    pub fn read_frame(&mut self) -> &[u8] {
        let frame = self.frames.pop_front().unwrap_or_default();
        &self.buffer[frame.starts_at..frame.ends_before]
    }
}

impl<const SIZE: usize> Default for FramedBuffer<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Default)]
struct BufferFrame {
    starts_at: usize,
    ends_before: usize,
}
