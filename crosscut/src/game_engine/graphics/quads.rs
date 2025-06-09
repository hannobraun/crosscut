#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Instance {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Instance {
    pub const MAX_NUM: u64 = 1024;
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        1 => Float32x3,
        2 => Float32x4,
    ];

    fn size() -> u64 {
        let Ok(size) = size_of::<Self>().try_into() else {
            unreachable!("Size of `Instance` can surely fit into a `u64`");
        };

        size
    }

    pub fn buffer_descriptor() -> wgpu::BufferDescriptor<'static> {
        wgpu::BufferDescriptor {
            label: None,
            size: Self::size() * Self::MAX_NUM,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES,
        }
    }
}
