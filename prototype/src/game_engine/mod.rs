mod game_engine;
mod terminal_editor;

pub use self::game_engine::{GameEngine, GameInput, GameOutput};

#[cfg(test)]
mod tests;
