mod editor;
mod game;
mod game_engine;
mod renderer;

pub use self::{
    editor::input::TerminalInput,
    game::{Game, PureCrosscutGame},
    game_engine::GameEngine,
    renderer::Renderer,
};

#[cfg(test)]
#[allow(unused)] // used only intermittently, to debug tests
pub use self::editor::output::codebase_to_stdout;

#[cfg(test)]
#[allow(unused)] // user was removed; but likely to become used again soon
pub use self::editor::output::codebase_to_string;
