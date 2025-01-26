pub const TILES_PER_AXIS: u8 = 32;

pub const PIXELS_PER_TILE_AXIS: usize = 8;
pub const PIXELS_PER_AXIS: usize =
    TILES_PER_AXIS as usize * PIXELS_PER_TILE_AXIS;
pub const NUM_PIXELS: usize = PIXELS_PER_AXIS * PIXELS_PER_AXIS;
pub const NUM_CHANNELS: usize = 4;
pub const NUM_PIXEL_BYTES: usize = NUM_PIXELS * NUM_CHANNELS;

pub fn set_pixel(
    tile_x: usize,
    tile_y: usize,
    color: [u8; 4],
    pixels: &mut [u8],
) {
    for offset_y in 0..PIXELS_PER_TILE_AXIS {
        for offset_x in 0..PIXELS_PER_TILE_AXIS {
            let pixel_x =
                (tile_x * PIXELS_PER_TILE_AXIS + offset_x) * NUM_CHANNELS;
            let pixel_y =
                (tile_y * PIXELS_PER_TILE_AXIS + offset_y) * NUM_CHANNELS;

            let i = pixel_y * PIXELS_PER_AXIS + pixel_x;
            pixels[i..i + NUM_CHANNELS].copy_from_slice(&color);
        }
    }
}
