mod editor;
mod input;
mod renderer;

pub use self::{editor::Editor, input::EditorInput, renderer::Renderer};

#[cfg(test)]
#[allow(unused)] // used sporadically, for debugging tests
pub fn render_code(
    code: &crate::language::code::Code,
    host: &crate::language::host::Host,
) {
    let mut renderer = Renderer::new(code, host, None);
    renderer.render_code().unwrap();
}
