use std::io::stdin;

use anyhow::anyhow;
use itertools::Itertools;

pub fn read_command() -> anyhow::Result<String> {
    let mut command = String::new();
    stdin().read_line(&mut command)?;
    Ok(command)
}

pub fn parse_command(
    command: String,
    color: &mut [f64; 4],
) -> anyhow::Result<()> {
    let Ok(channels) = command
        .split_whitespace()
        .map(|channel| channel.parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
    else {
        return Err(anyhow!("Can't parse color channels as `f64`."));
    };

    let Some((r, g, b, a)) = channels.into_iter().collect_tuple() else {
        return Err(anyhow!("Unexpected number of color channels."));
    };

    *color = [r, g, b, a];

    Ok(())
}
