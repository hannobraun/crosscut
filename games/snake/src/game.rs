use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crosscut::{
    Camera, Game, GameStart, Language, OrthographicProjection, Renderer,
    async_trait,
    glam::Vec2,
    wgpu,
    winit::{keyboard::Key, window::Window},
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
        let window_size = {
            let inner_size = window.inner_size();
            [inner_size.width, inner_size.height]
        };
        let camera = make_camera(window_size);

        Ok(Box::new(Snake {
            last_update: Instant::now(),
            position: Vec2::splat((WORLD_SIZE / 2.).floor()),
            camera,
            renderer: Renderer::new(window).await?,
        }))
    }
}

pub struct Snake {
    last_update: Instant,
    position: Vec2,
    camera: Camera,
    renderer: Renderer,
}

impl Game for Snake {
    fn on_code_update(&mut self, _: &mut Language) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_window_resized(&mut self, new_size: [u32; 2]) {
        self.camera = make_camera(new_size);
        self.renderer.handle_resize(new_size);
    }

    fn on_key(&mut self, key: Key) {
        let _ = key;
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let move_time = Duration::from_secs_f32(0.25);

        while self.last_update.elapsed() >= move_time {
            self.last_update += move_time;

            self.position[0] += 1.;
        }

        let position = [self.position.x, self.position.y, 0.];

        self.renderer.render(
            wgpu::Color {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            [position],
            &self.camera,
        )?;

        Ok(())
    }
}

fn make_camera(window_size: [u32; 2]) -> Camera {
    let world_min = -0.5;
    let world_max = WORLD_SIZE + world_min;

    let [window_width, window_height] = window_size.map(|size_u32| {
        let size_f32 = size_u32 as f32;
        assert_eq!(
            size_f32 as u32, size_u32,
            "Loss of precision while converting window size.",
        );

        size_f32
    });

    let far = -1.0;
    let near = 1.0;

    let projection = if window_width >= window_height {
        let width = WORLD_SIZE * window_width / window_height;
        let extra = (width - WORLD_SIZE) / 2.;

        OrthographicProjection {
            left: world_min - extra,
            right: world_max + extra,
            bottom: world_min,
            top: world_max,
            far,
            near,
        }
    } else {
        let height = WORLD_SIZE * window_height / window_width;
        let extra = (height - WORLD_SIZE) / 2.;

        OrthographicProjection {
            left: world_min,
            right: world_max,
            bottom: world_min - extra,
            top: world_max + extra,
            far,
            near,
        }
    };

    Camera::from_orthographic_projection(projection)
}

const WORLD_SIZE: f32 = 32.;
