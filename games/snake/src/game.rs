use std::sync::Arc;

use crosscut::{
    Camera, Game, Language, Renderer, async_trait, wgpu, winit::window::Window,
};

#[derive(Default)]
pub struct Snake {
    camera: Option<Camera>,
    renderer: Option<Renderer>,
}

#[async_trait]
impl Game for Snake {
    async fn on_start(
        &mut self,
        _: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<()> {
        self.camera = Some(Camera::default());
        self.renderer = Some(Renderer::new(window).await?);
        Ok(())
    }

    fn on_code_update(&mut self, _: &mut Language) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let (Some(camera), Some(renderer)) = (&self.camera, &self.renderer)
        else {
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
            camera,
        )?;

        Ok(())
    }
}
