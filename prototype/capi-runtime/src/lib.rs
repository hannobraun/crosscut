use std::{mem, slice};

#[no_mangle]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[no_mangle]
pub extern "C" fn allocate_draw_buffer(
    canvas_width: usize,
    canvas_height: usize,
) -> *mut u8 {
    const NUM_COLOR_CHANNELS: usize = 4;
    let len = canvas_width * canvas_height * NUM_COLOR_CHANNELS;

    let mut buffer = Vec::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    mem::forget(buffer);

    ptr
}

#[no_mangle]
pub extern "C" fn draw_cell(
    cell_size: usize,
    base_i: usize,
    base_j: usize,
    color: u8,
    buffer: *mut u8,
    buffer_length: usize,
    width: usize,
) {
    assert!(!buffer.is_null());

    let array = unsafe { slice::from_raw_parts_mut(buffer, buffer_length) };

    for i in 0..cell_size {
        for j in 0..cell_size {
            let abs_i = base_i + i;
            let abs_j = base_j + j;

            let index = abs_i + abs_j * width;
            array[index] = color;
        }
    }
}
