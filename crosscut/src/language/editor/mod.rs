mod editor;
mod input_buffer;
mod input_event;
mod layout;

pub use self::{
    editor::{Editor, EditorCommand},
    input_buffer::EditorInputBuffer,
    input_event::EditorInput,
    layout::{EditorLayout, EditorLine},
};

#[cfg(test)]
mod tests;
