use std::sync::Arc;

use crosscut::{
    Camera, Game, GameStart, Language, OrthographicProjection, Renderer,
    async_trait, wgpu, winit::window::Window,
};

#[derive(Default)]
pub struct SnakeStart {}

#[async_trait]
impl GameStart for SnakeStart {
    async fn on_start(
        &mut self,
        _: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<Box<dyn Game>> {
        let world_size = 32.;
        let world_min = -0.5;
        let world_max = world_size + world_min;

        let [window_width, window_height] = {
            let window_size = window.inner_size();

            [window_size.width, window_size.height].map(|size_u32| {
                let size_f32 = size_u32 as f32;
                assert_eq!(
                    size_f32 as u32, size_u32,
                    "Loss of precision while converting window size.",
                );

                size_f32
            })
        };

        let far = -1.0;
        let near = 1.0;

        let projection = if window_width >= window_height {
            let width = world_size * window_width / window_height;
            let extra = (width - world_size) / 2.;

            OrthographicProjection {
                left: world_min - extra,
                right: world_max + extra,
                bottom: world_min,
                top: world_max,
                far,
                near,
            }
        } else {
            let height = world_size * window_height / window_width;
            let extra = (height - world_size) / 2.;

            OrthographicProjection {
                left: world_min,
                right: world_max,
                bottom: world_min - extra,
                top: world_max + extra,
                far,
                near,
            }
        };

        Ok(Box::new(Snake {
            camera: Camera::from_orthographic_projection(projection),
            renderer: Some(Renderer::new(window).await?),
        }))
    }
}

pub struct Snake {
    camera: Camera,
    renderer: Option<Renderer>,
}

impl Game for Snake {
    fn on_code_update(&mut self, _: &mut Language) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let Some(renderer) = &self.renderer else {
            return Ok(());
        };

        renderer.render(
            wgpu::Color {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            [[0., 0., 0.]],
            &self.camera,
        )?;

        Ok(())
    }
}
