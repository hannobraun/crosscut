use std::{io::stdin, thread::JoinHandle};

use anyhow::anyhow;
use itertools::Itertools;

use crate::{
    actor::{Actor, Sender},
    language::Command,
};

pub fn start(commands: Sender<Command>) -> JoinHandle<()> {
    Actor::spawn(move |command| {
        let command = match parse_command(command) {
            Ok(command) => command,
            Err(err) => {
                println!("{err}");
                return true;
            }
        };

        commands.send(command).is_ok()
    })
    .provide_input(|| {
        let mut command = String::new();
        stdin().read_line(&mut command).unwrap();

        command
    })
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
