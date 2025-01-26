use std::cell::UnsafeCell;

/// # Data that is shared with the JavaScript host
///
/// This type is designed to be used in a `static`, so the use of `static mut`
/// can be avoided.
///
/// ## Safety
///
/// This type is designed for use in WebAssembly. It is _unsound_ to use it in a
/// multi-threaded context.
#[repr(transparent)]
pub struct Shared<T> {
    inner: UnsafeCell<T>,
}

impl<T> Shared<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }

    /// # Gain access to the shared data
    ///
    /// ## `&self` argument
    ///
    /// This method returns a mutable reference, despite only requiring `&self`.
    /// This is fine, as the method is `unsafe` and the requirements that derive
    /// from this are documented.
    ///
    /// If this took `&mut self`, then `Shared` would need to live in a
    /// `static mut`, which would have the same pitfalls, and more. With the
    /// current design, `Shared` can live in a non-`mut` `static`.
    ///
    /// ## Safety
    ///
    /// The caller must drop the returned reference before giving back control
    /// to the JavaScript host.
    ///
    /// The caller must not call this method again, while the returned reference
    /// still exists.
    #[allow(clippy::mut_from_ref)] // function is `unsafe` and well-documented
    pub unsafe fn access(&self) -> &mut T {
        &mut *self.inner.get()
    }
}

// Safe to implement, since WebAssembly code runs in a single-threaded context.
unsafe impl<T> Sync for Shared<T> {}
