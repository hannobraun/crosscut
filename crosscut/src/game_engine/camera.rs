use glam::Mat4;

pub struct Camera {
    transform: [[f32; 4]; 4],
}

impl Camera {
    pub fn from_orthographic_projection(ortho: OrthographicProjection) -> Self {
        let transform = Mat4::orthographic_rh(
            ortho.left,
            ortho.right,
            ortho.bottom,
            ortho.top,
            ortho.near,
            ortho.far,
        );

        Self {
            transform: transform.to_cols_array_2d(),
        }
    }

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

pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub far: f32,
    pub near: f32,
}
