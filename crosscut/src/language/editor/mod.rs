mod editor;
mod input;
mod input_buffer;
mod layout;

pub use self::{
    editor::{Editor, EditorCommand},
    input::EditorInput,
    input_buffer::EditorInputBuffer,
    layout::{EditorLayout, EditorLine},
};

#[cfg(test)]
mod tests;
