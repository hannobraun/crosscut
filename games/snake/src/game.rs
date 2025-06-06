use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use crosscut::{
    Camera, Game, GameStart, Language, OrthographicProjection, Renderer,
    async_trait,
    glam::{Vec2, Vec3},
    wgpu,
    winit::{keyboard::KeyCode, window::Window},
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
            positions: VecDeque::from([Vec2::splat((WORLD_SIZE / 2.).floor())]),
            velocity: Vec2::new(1., 0.),
            camera,
            renderer: Renderer::new(window).await?,
        }))
    }
}

pub struct Snake {
    last_update: Instant,
    positions: VecDeque<Vec2>,
    velocity: Vec2,
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

    fn on_key(&mut self, key: KeyCode) {
        self.velocity = match key {
            KeyCode::ArrowUp | KeyCode::KeyW => Vec2::new(0., 1.),
            KeyCode::ArrowLeft | KeyCode::KeyA => Vec2::new(-1., 0.),
            KeyCode::ArrowDown | KeyCode::KeyS => Vec2::new(0., -1.),
            KeyCode::ArrowRight | KeyCode::KeyD => Vec2::new(1., 0.),
            _ => {
                return;
            }
        };
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let move_time = Duration::from_secs_f32(0.1);

        while self.last_update.elapsed() >= move_time {
            self.last_update += move_time;

            let Some(head) = self.positions.front().copied() else {
                unreachable!("The body is never empty.");
            };

            self.positions.pop_back();

            self.positions.push_front(head + self.velocity);
        }

        let positions = self
            .positions
            .iter()
            .map(|position| Vec3::new(position.x, position.y, 0.));

        self.renderer.render(
            wgpu::Color {
                r: 1.,
                g: 1.,
                b: 1.,
                a: 1.,
            },
            positions,
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
