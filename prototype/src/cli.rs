use std::io::stdin;

use itertools::Itertools;

pub fn read_command() -> anyhow::Result<String> {
    let mut command = String::new();
    stdin().read_line(&mut command)?;
    Ok(command)
}

pub fn parse_command(command: String, color: &mut [f64; 4]) {
    let Ok(channels) = command
        .split_whitespace()
        .map(|channel| channel.parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
    else {
        println!("Can't parse color channels as `f64`.");
        return;
    };

    let Some((r, g, b, a)) = channels.into_iter().collect_tuple() else {
        println!("Unexpected number of color channels.");
        return;
    };

    *color = [r, g, b, a];
}
