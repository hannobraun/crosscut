mod compiler;
mod debugger;
mod display;
mod ffi;
mod games;
mod source_map;
mod state;
mod syntax;
mod tiles;
mod updates;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Error)
        .expect("Failed to initialize logging to console");
}
