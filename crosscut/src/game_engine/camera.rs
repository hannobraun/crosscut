#[derive(Default)]
pub struct Camera {}

impl Camera {
    pub fn to_transform(&self) -> [[f32; 4]; 4] {
        [
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ]
    }
}
