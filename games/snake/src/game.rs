use std::sync::Arc;

use crosscut::{
    Game, Language, Renderer, async_trait, wgpu, winit::window::Window,
};

#[derive(Default)]
pub struct Snake {
    renderer: Option<Renderer>,
}

#[async_trait]
impl Game for Snake {
    async fn on_start(
        &mut self,
        _: &mut Language,
        window: &Arc<Window>,
    ) -> anyhow::Result<()> {
        self.renderer = Some(Renderer::new(window).await?);
        Ok(())
    }

    fn on_code_update(&mut self, _: &mut Language) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_frame(&mut self, _: &mut Language) -> anyhow::Result<()> {
        let Some(renderer) = &self.renderer else {
            return Ok(());
        };

        renderer.render(wgpu::Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        })?;

        Ok(())
    }
}
