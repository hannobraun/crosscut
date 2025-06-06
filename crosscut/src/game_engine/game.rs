use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use winit::{keyboard::KeyCode, window::Window};

use crate::{
    game_engine::Renderer,
    language::{
        code::Type,
        language::Language,
        runtime::{Effect, RuntimeState, Value},
    },
};

use super::Camera;

#[async_trait]
pub trait GameStart {
    async fn on_start(
        &mut self,
        language: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<Box<dyn Game>>;
}

pub trait Game {
    fn on_code_update(&mut self, language: &mut Language)
    -> anyhow::Result<()>;

    fn on_window_resized(&mut self, new_size: [u32; 2]);

    fn on_key(&mut self, key: KeyCode);

    fn on_frame(&mut self, language: &mut Language) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct PureCrosscutGameStart {}

#[async_trait]
impl GameStart for PureCrosscutGameStart {
    async fn on_start(
        &mut self,
        _: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<Box<dyn Game>> {
        Ok(Box::new(PureCrosscutGame {
            state: State::Running,
            renderer: Renderer::new(window).await?,
            color: wgpu::Color::BLACK,
        }))
    }
}

pub struct PureCrosscutGame {
    state: State,
    renderer: Renderer,
    color: wgpu::Color,
}

impl Game for PureCrosscutGame {
    fn on_code_update(
        &mut self,
        language: &mut Language,
    ) -> anyhow::Result<()> {
        self.run_game_for_a_few_steps(language)?;
        Ok(())
    }

    fn on_window_resized(&mut self, new_size: [u32; 2]) {
        self.renderer.handle_resize(new_size);
    }

    fn on_key(&mut self, key: KeyCode) {
        let _ = key;
    }

    fn on_frame(&mut self, language: &mut Language) -> anyhow::Result<()> {
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

                    language.exit_from_provided_function(Value::nothing());
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

        self.run_game_for_a_few_steps(language)?;

        self.renderer.render(self.color, [], &Camera::default())?;

        Ok(())
    }
}

impl PureCrosscutGame {
    fn run_game_for_a_few_steps(
        &mut self,
        language: &mut Language,
    ) -> anyhow::Result<()> {
        if let State::WaitUntil { instant } = self.state {
            if Instant::now() < instant {
                return Ok(());
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

                    language.exit_from_provided_function(Value::nothing());
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

        let max_steps = 1024;

        for _ in 0..max_steps {
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

                                        self.color = wgpu::Color {
                                            r: value,
                                            g: value,
                                            b: value,
                                            a: 1.,
                                        };

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

        Ok(())
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
