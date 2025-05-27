// It makes sense to prevent this in public APIs, but it also warns me about the
// names of private modules that I only re-export from. That provides no value
// and is pretty annoying.
#![allow(clippy::module_inception)]

pub mod game_engine;
pub mod io;
pub mod language;
pub mod threads;
pub mod util;
