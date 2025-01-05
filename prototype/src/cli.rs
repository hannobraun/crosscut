use std::{
    io::stdin,
    sync::mpsc::{self, SendError},
    thread,
};

use anyhow::anyhow;
use itertools::Itertools;

use crate::{actor::Sender, language::Command};

pub fn start(commands: Sender<Command>) {
    let (commands_tx, commands_rx) = mpsc::channel();

    thread::spawn(move || loop {
        let mut command = String::new();
        stdin().read_line(&mut command).unwrap();

        if let Err(SendError(_)) = commands_tx.send(command) {
            break;
        }
    });

    thread::spawn(move || loop {
        let Ok(command) = commands_rx.recv() else {
            break;
        };
        let Some(command) = read_command(command) else {
            continue;
        };

        if let Err(SendError(_)) = commands.send(command) {
            // The other end has hung up. We should shut down too.
            break;
        }
    });
}

fn read_command(command: String) -> Option<Command> {
    let command = match parse_command(command) {
        Ok(command) => command,
        Err(err) => {
            println!("{err}");
            return None;
        }
    };

    Some(command)
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
