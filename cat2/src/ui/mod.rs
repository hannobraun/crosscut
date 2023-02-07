mod area;
mod border;
mod buffer;
mod generations;
mod vector;

pub use self::buffer::Buffer;

use std::io::Stdout;

use crossterm::terminal;

use crate::cells::{self, Generation};

use self::vector::Vector;

pub fn draw(
    generations: &[Generation],
    buffer: &mut Buffer,
    stdout: &mut Stdout,
) -> anyhow::Result<()> {
    let (num_columns, num_rows) = terminal::size()?;
    buffer.prepare(Vector {
        x: num_columns as usize,
        y: num_rows as usize,
    });

    let generations_width = cells::NUM_CELLS + 2;

    let offset = Vector {
        x: num_columns as usize - generations_width,
        y: 0,
    };
    let size = Vector {
        x: generations_width,
        y: num_rows as usize,
    };
    let area = area::new(buffer, offset, size);

    generations::draw(area, generations.iter());

    buffer.draw(stdout)?;

    Ok(())
}
