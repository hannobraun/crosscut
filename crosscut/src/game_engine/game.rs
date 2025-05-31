use std::time::{Duration, Instant};

use crate::language::{
    code::Type,
    language::Language,
    runtime::{Effect, RuntimeState, Value},
};

use super::GameOutput;

pub trait Game {
    fn on_start(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    );

    fn on_editor_input(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    );

    fn on_frame(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    );
}

#[derive(Default)]
pub struct PureCrosscutGame {
    state: State,
}

impl PureCrosscutGame {
    fn run_game_for_a_few_steps(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    ) {
        if let State::WaitUntil { instant } = self.state {
            if Instant::now() < instant {
                return;
            }

            match language.evaluator().state() {
                RuntimeState::Effect {
                    effect: Effect::ApplyProvidedFunction { name, input: _ },
                    ..
                } => {
                    assert_eq!(
                        name, "sleep_ms",
                        "Expecting to provide output for `sleep_ms` function, \
                        because that is the only one that enters this state.",
                    );

                    language.provide_host_function_output(Value::nothing());
                }
                state => {
                    assert!(
                        matches!(state, RuntimeState::Started),
                        "`WaitUntil` state was entered, but expected effect is \
                        not active. This should only happen, if the runtime \
                        has been reset.",
                    );
                }
            }

            self.state = State::Running;
        }

        let mut num_steps = 0;

        loop {
            num_steps += 1;
            if num_steps > 1024 {
                break;
            }

            match language.step().clone() {
                RuntimeState::Started | RuntimeState::Running => {
                    continue;
                }
                RuntimeState::Effect { effect, .. } => {
                    match effect {
                        Effect::ApplyProvidedFunction { name, input } => {
                            match name.as_str() {
                                "color" => match input {
                                    Value::Integer { value } => {
                                        let value: f64 = value.into();
                                        let value = value / 255.;

                                        output.push(GameOutput::SubmitColor {
                                            color: [value, value, value, 1.],
                                        });

                                        self.state = State::EndOfFrame;
                                        break;
                                    }
                                    value => {
                                        language.trigger_effect(
                                            Effect::UnexpectedInput {
                                                expected: Type::Integer,
                                                actual: value,
                                            },
                                        );
                                    }
                                },
                                "sleep_ms" => match input {
                                    Value::Integer { value } if value >= 0 => {
                                        let value = value as u64;

                                        self.state = State::WaitUntil {
                                            instant: Instant::now()
                                                + Duration::from_millis(value),
                                        };
                                        break;
                                    }
                                    value => {
                                        language.trigger_effect(
                                            Effect::UnexpectedInput {
                                                expected: Type::Integer,
                                                actual: value,
                                            },
                                        );
                                    }
                                },
                                _ => {
                                    language.trigger_effect(
                                        Effect::ProvidedFunctionNotFound,
                                    );
                                }
                            };
                            continue;
                        }
                        _ => {
                            // We can't handle any other effect.
                            break;
                        }
                    }
                }
                RuntimeState::Finished { output } => {
                    if let Ok(body) = output.into_function_body() {
                        // If the program returns a function, we call that.
                        //
                        // Eventually, we would want something more stringent
                        // here, like expect a `main` function, or a module in a
                        // specific format. For now, this will do though.
                        language.apply_function(body);
                        continue;
                    }
                }
            }

            break;
        }
    }
}

impl Game for PureCrosscutGame {
    fn on_start(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    ) {
        self.run_game_for_a_few_steps(language, output);
    }

    fn on_editor_input(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    ) {
        self.run_game_for_a_few_steps(language, output);
    }

    fn on_frame(
        &mut self,
        language: &mut Language,
        output: &mut Vec<GameOutput>,
    ) {
        if let State::EndOfFrame = self.state {
            match language.evaluator().state() {
                RuntimeState::Effect {
                    effect: Effect::ApplyProvidedFunction { name, input: _ },
                    ..
                } => {
                    assert_eq!(
                        name, "color",
                        "Expecting to provide output for `color` function, \
                        because that is the only one that enters this state.",
                    );

                    language.provide_host_function_output(Value::nothing());
                }
                state => {
                    assert!(
                        matches!(state, RuntimeState::Started),
                        "`EndOfFrame` state was entered, but expected effect \
                        is not active. This should only happen, if the runtime \
                        has been reset.",
                    );
                }
            }

            self.state = State::Running;
        }

        self.run_game_for_a_few_steps(language, output);
    }
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Running,

    EndOfFrame,
    WaitUntil {
        instant: Instant,
    },
}
