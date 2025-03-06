mod editor;
mod game_engine;

pub use self::{
    editor::input::TerminalInputEvent,
    game_engine::{GameEngine, GameInput, GameOutput},
};

#[cfg(test)]
#[allow(unused)] // used only intermittently, to debug tests
pub use self::editor::output::codebase_to_stdout;

#[cfg(test)]
mod tests;
