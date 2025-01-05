use std::{io::stdin, thread};

use anyhow::anyhow;
use itertools::Itertools;
use tokio::sync::mpsc::{self, error::SendError, UnboundedReceiver};

pub fn start() -> UnboundedReceiver<String> {
    let (commands_tx, commands_rx) = mpsc::unbounded_channel();

    // We're using Tokio here and could use its asynchronous stdio API. But the
    // Tokio documentation explicitly recommends against using that for
    // interactive code, recommending a dedicated thread instead.
    thread::spawn(move || loop {
        let command = read_command().unwrap();
        if let Err(SendError(_)) = commands_tx.send(command) {
            // The other end has hung up. We should shut down too.
            break;
        }
    });

    commands_rx
}

pub fn read_command() -> anyhow::Result<String> {
    let mut command = String::new();
    stdin().read_line(&mut command)?;
    Ok(command)
}

pub fn parse_command(command: String) -> anyhow::Result<Command> {
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

    Ok(Command::SetColor {
        color: [r, g, b, a],
    })
}

pub enum Command {
    SetColor { color: [f64; 4] },
}
