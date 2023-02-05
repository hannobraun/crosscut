use super::area::{self, Area};

pub fn print_horizontal(
    area: &mut Area,
    left_corner: &str,
    right_corner: &str,
    width: u16,
) -> anyhow::Result<()> {
    area::write(area, left_corner)?;
    (0..width).try_for_each(|_| area::write(area, "━"))?;
    area::write(area, right_corner)?;

    Ok(())
}

pub fn print_vertical(area: &mut Area) -> anyhow::Result<()> {
    area::write(area, "┃")
}
