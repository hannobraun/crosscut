use crate::actor::{actor, Sender};

pub fn start(
    color: Sender<[f64; 4]>,
) -> anyhow::Result<(Sender<GameInput>, Sender<Command>)> {
    let mut code = Code {
        color: [0., 0., 0., 1.],
    };

    println!("Color: {:?}", code.color);

    let handle_events = actor(move |event| {
        match event {
            Event::Command(Command::SetColor { color }) => {
                code.color = color;
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        color.send(code.color).is_ok()
    });

    let events_from_input = handle_events.input.clone();
    let input_to_event = actor(move |input| {
        events_from_input.send(Event::GameInput(input)).is_ok()
    });

    let events_from_commands = handle_events.input;
    let command_to_event = actor(move |command| {
        events_from_commands.send(Event::Command(command)).is_ok()
    });

    Ok((input_to_event.input, command_to_event.input))
}

struct Code {
    color: [f64; 4],
}

enum Event {
    Command(Command),
    GameInput(GameInput),
}

pub enum Command {
    SetColor { color: [f64; 4] },
}

pub enum GameInput {
    RenderingFrame,
}
