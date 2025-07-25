use std::{
    array,
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use crosscut::{
    Camera, Game, Init, Instance, Language, OrthographicProjection, Renderer,
    async_trait,
    glam::Vec2,
    wgpu,
    winit::{keyboard::KeyCode, window::Window},
};
use rand::random;

#[derive(Default)]
pub struct TrialOfTheCaterpillarInit {}

#[async_trait]
impl Init for TrialOfTheCaterpillarInit {
    fn name(&self) -> Option<&str> {
        Some("Trial of the Caterpillar")
    }

    async fn init(
        &mut self,
        _: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<Box<dyn Game>> {
        let window_size = {
            let inner_size = window.inner_size();
            [inner_size.width, inner_size.height]
        };
        let camera = make_camera(window_size);
        let renderer = Renderer::new(window).await?;

        Ok(Box::new(TrialOfTheCaterpillar {
            last_update: Instant::now(),
            world: World::new(),
            camera,
            renderer,
        }))
    }
}

pub struct TrialOfTheCaterpillar {
    last_update: Instant,
    world: World,
    camera: Camera,
    renderer: Renderer,
}

impl Game for TrialOfTheCaterpillar {
    fn on_code_update(&mut self, _: &mut Language) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_window_resized(&mut self, new_size: [u32; 2]) {
        self.camera = make_camera(new_size);
        self.renderer.handle_resize(new_size);
    }

    fn on_key(&mut self, key: KeyCode) {
        let new_velocity = match key {
            KeyCode::ArrowUp | KeyCode::KeyW => Vec2::new(0., 1.),
            KeyCode::ArrowLeft | KeyCode::KeyA => Vec2::new(-1., 0.),
            KeyCode::ArrowDown | KeyCode::KeyS => Vec2::new(0., -1.),
            KeyCode::ArrowRight | KeyCode::KeyD => Vec2::new(1., 0.),
            _ => {
                return;
            }
        };

        let is_valid = match self.world.input.back().copied() {
            Some(latest_input) if new_velocity * -1. != latest_input => true,
            Some(_) => false,
            None => true,
        };

        if is_valid {
            self.world.input.push_back(new_velocity);
        }
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let move_time = Duration::from_secs_f32(0.1);

        while self.last_update.elapsed() >= move_time {
            self.last_update += move_time;

            self.world.update();
        }

        let positions = self
            .world
            .snake
            .iter()
            .map(|position| Instance {
                position: [position.x, position.y, 0.],
                color: [0., 1., 0., 1.],
            })
            .chain(self.world.walls.iter().map(|position| Instance {
                position: [position.x, position.y, 0.],
                color: [1., 1., 1., 1.],
            }))
            .chain(self.world.food.map(|position| Instance {
                position: [position.x, position.y, 0.],
                color: [1., 0., 0., 1.],
            }))
            .chain(self.world.new_walls.iter().map(|position| Instance {
                position: [position.x, position.y, 0.],
                color: [0.5, 1., 0.5, 1.],
            }));

        self.renderer.render(
            wgpu::Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
            positions,
            [],
            &self.camera,
        )?;

        Ok(())
    }
}

struct World {
    input: VecDeque<Vec2>,
    walls: Vec<Vec2>,
    snake: VecDeque<Vec2>,
    nominal_length: usize,
    velocity: Vec2,
    food: Option<Vec2>,
    new_walls: VecDeque<Vec2>,
    new_walls_left: usize,
}

impl World {
    pub fn new() -> Self {
        Self {
            input: VecDeque::new(),
            walls: make_walls(),
            snake: VecDeque::from([Vec2::splat((WORLD_SIZE / 2.).floor())]),
            nominal_length: 3,
            velocity: Vec2::new(1., 0.),
            food: None,
            new_walls: VecDeque::new(),
            new_walls_left: 0,
        }
    }

    fn update(&mut self) {
        self.process_input();
        self.spawn_food();
        self.move_snake();
        self.eat_food();
        self.collide_snake_with_walls();
        self.collide_snake_with_itself();
        self.finish_new_walls();
    }

    fn process_input(&mut self) {
        if let Some(velocity) = self.input.pop_front() {
            self.velocity = velocity;
        }
    }

    fn spawn_food(&mut self) {
        if self.food.is_none() {
            loop {
                let [x, y] = array::from_fn(|_| {
                    (random::<f32>() * (WORLD_SIZE)).floor()
                });
                let position = Vec2::new(x, y);

                if collision_between(&position, &self.walls) {
                    continue;
                }

                self.food = Some(position);
                break;
            }
        }
    }

    fn move_snake(&mut self) {
        let Some(head) = self.snake.front().copied() else {
            unreachable!("The body is never empty.");
        };

        if self.snake.len() >= self.nominal_length {
            self.snake.pop_back();
        }

        self.snake.push_front(head + self.velocity);

        if self.new_walls_left > 0 {
            self.new_walls.push_back(head);
            self.new_walls_left -= 1;
        }
    }

    fn eat_food(&mut self) {
        if let Some(food) = self.food {
            if collision_between(&food, &self.snake) {
                self.food = None;
                self.nominal_length += 3;

                self.new_walls_left += 3;
            }
        }
    }

    fn collide_snake_with_walls(&mut self) {
        let Some(head) = self.snake.front() else {
            unreachable!("There is always a snake head.");
        };

        if collision_between(head, &self.walls) {
            *self = Self::new();
        }
    }

    fn collide_snake_with_itself(&mut self) {
        if let Some(head) = self.snake.front() {
            let body = self.snake.iter().skip(1);
            if collision_between(head, body) {
                *self = Self::new();
            }
        }
    }

    fn finish_new_walls(&mut self) {
        if let Some(new_wall) = self.new_walls.pop_front() {
            let body = self.snake.iter().skip(1);
            if collision_between(&new_wall, body) {
                self.new_walls.push_front(new_wall);
            } else {
                self.walls.push(new_wall);
            }
        }
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

fn make_walls() -> Vec<Vec2> {
    let mut walls = Vec::new();

    for x in [0., WORLD_SIZE - 1.] {
        let mut y = 0.;

        while y < WORLD_SIZE {
            walls.push(Vec2::new(x, y));
            y += 1.
        }
    }

    for y in [0., WORLD_SIZE - 1.] {
        let mut x = 1.;

        while x < WORLD_SIZE - 1. {
            walls.push(Vec2::new(x, y));
            x += 1.;
        }
    }

    walls
}

fn collision_between<'a>(
    a: &Vec2,
    b: impl IntoIterator<Item = &'a Vec2>,
) -> bool {
    let mut collision = false;

    for p in b {
        collision |= a == p;
    }

    collision
}

const WORLD_SIZE: f32 = 32.;
