mod editor;
mod game_engine;

pub use self::{
    editor::input::TerminalInputEvent,
    game_engine::{GameEngine, GameInput, GameOutput},
};

#[cfg(test)]
mod tests;
