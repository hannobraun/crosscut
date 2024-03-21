use core::cell::UnsafeCell;

pub const DATA_SIZE: usize = 2usize.pow(24); // 16 MiB
pub static DATA: SharedMemory<DATA_SIZE> = SharedMemory::new();

#[no_mangle]
pub extern "C" fn data_ptr() -> usize {
    &DATA as *const _ as usize
}

#[no_mangle]
pub extern "C" fn data_len() -> usize {
    DATA_SIZE
}

#[no_mangle]
pub extern "C" fn on_init() {
    // Sound, as we only call this once and only use it here locally.
    let data = unsafe { DATA.access_write() };

    for chunk in data.chunks_mut(4) {
        chunk[0] = 0;
        chunk[1] = 255;
        chunk[2] = 0;
        chunk[3] = 255;
    }

    println!("Caterpillar initialized.");
}

/// # Caterpillar memory that is shared with the JavaScript host
///
/// ## Safety
///
/// We are in a single-threaded context. Shared memory is only accessed by top-
/// level FFI functions in this module and the JavaScript host. Since neither of
/// those can run concurrently, this doesn't constitute concurrent access.
///
/// As a consequence, access is sound, as long as no reference to this static
/// lives longer than the local scope of the FFI function that creates it.
#[repr(transparent)]
pub struct SharedMemory<const SIZE: usize> {
    inner: UnsafeCell<[u8; SIZE]>,
}

impl<const SIZE: usize> SharedMemory<SIZE> {
    const fn new() -> Self {
        Self {
            inner: UnsafeCell::new([0; SIZE]),
        }
    }

    /// # Gain write access to the shared memory
    ///
    /// This method is private, to prevent any access within Rust code that
    /// doesn't come from the top-level FFI functions.
    ///
    /// ## `&self` argument
    ///
    /// This method returns a mutable reference, despite only requiring `&self`.
    /// This is fine, as the method is `unsafe` and the requirements that come
    /// from this are documented.
    ///
    /// If this took `&mut self`, the `SharedMemory` would need to live in a
    /// `static mut`, which would have the same pitfalls, and more. With the
    /// current design, `SharedMemory` can live in a non-`mut` `static`.
    ///
    /// ## Safety
    ///
    /// The caller must drop the returned reference before returning control to
    /// the JavaScript host.
    ///
    /// The caller must not call [`SharedMemory::access_write`] again, while the
    /// returned reference still exists.
    #[allow(clippy::mut_from_ref)] // function is `unsafe` and well-documented
    unsafe fn access_write(&self) -> &mut [u8] {
        &mut *self.inner.get()
    }
}

// Safe to implement, since with WebAssembly, this lives in a single-threaded
// context.
unsafe impl<const SIZE: usize> Sync for SharedMemory<SIZE> {}
