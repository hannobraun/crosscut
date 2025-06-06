pub struct Camera {
    transform: [[f32; 4]; 4],
}

impl Camera {
    pub fn to_transform(&self) -> [[f32; 4]; 4] {
        self.transform
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            transform: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
        }
    }
}
