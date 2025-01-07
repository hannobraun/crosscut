use crate::{
    actor::{Actor, Sender, ThreadHandle},
    code::{Code, Expression},
    editor,
    interpreter::Interpreter,
};

pub fn start(
    game_output: Sender<GameOutput>,
) -> anyhow::Result<(ThreadHandle, Actor<String>, Actor<GameInput>)> {
    let mut code = Code::default();
    let mut interpreter = Interpreter::default();
    let mut current_function = None;

    editor::update(&code, &interpreter)?;

    let handle_events = Actor::spawn(move |event| {
        match event {
            Event::EditorInput { line } => {
                let expressions = parse(line);
                code.expressions.extend(expressions);
                editor::update(&code, &interpreter)?;
            }
            Event::GameInput(GameInput::RenderingFrame) => {
                // This loop is coupled to the frame rate of the renderer.
            }
        }

        if let Some(expression) =
            code.expressions.get(interpreter.next_expression)
        {
            match expression {
                Expression::Identifier { name } => {
                    if name == "submit_color" && current_function.is_none() {
                        current_function = Some(HostFunction::SubmitColor);
                        interpreter.next_expression += 1;
                    }
                }
                Expression::LiteralNumber { value } => {
                    if let Some(function) = current_function {
                        match function {
                            HostFunction::SubmitColor => {
                                current_function =
                                    Some(HostFunction::SubmitColorR {
                                        r: *value,
                                    });
                            }
                            HostFunction::SubmitColorR { r } => {
                                current_function =
                                    Some(HostFunction::SubmitColorRG {
                                        r,
                                        g: *value,
                                    });
                            }
                            HostFunction::SubmitColorRG { r, g } => {
                                current_function =
                                    Some(HostFunction::SubmitColorRGB {
                                        r,
                                        g,
                                        b: *value,
                                    });
                            }
                            HostFunction::SubmitColorRGB { r, g, b } => {
                                current_function = None;
                                game_output.send(GameOutput::SubmitColor {
                                    color: [r, g, b, *value],
                                })?;
                            }
                        }

                        interpreter.next_expression += 1;
                    }
                }
            }
        }

        Ok(())
    });

    let events_from_commands = handle_events.sender.clone();
    let handle_editor_input = Actor::spawn(move |line| {
        events_from_commands.send(Event::EditorInput { line })?;
        Ok(())
    });

    let events_from_input = handle_events.sender;
    let input_to_event = Actor::spawn(move |input| {
        events_from_input.send(Event::GameInput(input))?;
        Ok(())
    });

    Ok((handle_events.handle, handle_editor_input, input_to_event))
}

enum Event {
    EditorInput { line: String },
    GameInput(GameInput),
}

pub enum GameInput {
    RenderingFrame,
}

pub enum GameOutput {
    SubmitColor { color: [f64; 4] },
}

fn parse(line: String) -> Vec<Expression> {
    line.split_whitespace()
        .map(|token| match token.parse::<f64>() {
            Ok(value) => Expression::LiteralNumber { value },
            Err(_) => Expression::Identifier {
                name: token.to_string(),
            },
        })
        .collect()
}

#[derive(Clone, Copy)]
enum HostFunction {
    SubmitColor,
    SubmitColorR { r: f64 },
    SubmitColorRG { r: f64, g: f64 },
    SubmitColorRGB { r: f64, g: f64, b: f64 },
}
