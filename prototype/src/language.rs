use crate::actor::{Actor, ActorHandle, Sender};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ActorHandle, Actor<Command>, Actor<GameInput>)> {
    let mut code = Code {
        color: [0., 0., 0., 1.],
    };

    print_output(&code);

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::Command(Command::SetColor { color }) => {
                code.color = color;
                print_output(&code);
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        game_output.send(GameOutput::SubmitColor { color: code.color })?;

        Ok(())
    });

    let events_from_commands = handle_events.sender.clone();
    let command_to_event = Actor::spawn(move |command| {
        events_from_commands.send(Event::Command(command))?;
        Ok(())
    });

    let events_from_input = handle_events.sender;
    let input_to_event = Actor::spawn(move |input| {
        events_from_input.send(Event::GameInput(input))?;
        Ok(())
    });

    Ok((handle_events.handle, command_to_event, input_to_event))
}

pub struct Code {
    pub color: [f64; 4],
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

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}

fn print_output(code: &Code) {
    println!("Color: {:?}", code.color);
}
