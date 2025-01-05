use std::{io::stdin, sync::mpsc::SendError, thread};

use anyhow::anyhow;
use itertools::Itertools;

use crate::actor::Sender;

pub fn start(commands: Sender<Command>) {
    thread::spawn(move || loop {
        let Some(command) = read_command().unwrap() else {
            continue;
        };

        if let Err(SendError(_)) = commands.send(command) {
            // The other end has hung up. We should shut down too.
            break;
        }
    });
}

fn read_command() -> anyhow::Result<Option<Command>> {
    let mut command = String::new();
    stdin().read_line(&mut command)?;

    let command = match parse_command(command) {
        Ok(command) => command,
        Err(err) => {
            println!("{err}");
            return Ok(None);
        }
    };

    Ok(Some(command))
}

fn parse_command(command: String) -> anyhow::Result<Command> {
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
