use crate::{
    actor::{Actor, ActorHandle, Sender},
    code::Code,
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ActorHandle, Actor<Command>, Actor<GameInput>)> {
    let mut code = Code {
        expressions: vec![],
    };

    print_output(&code);

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::Command(Command::SetColor { color }) => {
                code.expressions = vec![color];
                print_output(&code);
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        for &color in &code.expressions {
            game_output.send(GameOutput::SubmitColor { color })?;
        }

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
    println!("Color: {:?}", code.expressions);
}
