#![no_std]

extern crate alloc;

mod ffi_in;
mod ffi_out;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: lol_alloc::LockedAllocator<lol_alloc::FreeListAllocator> =
    lol_alloc::LockedAllocator::new(lol_alloc::FreeListAllocator::new());

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use alloc::string::ToString;
    let msg = info.to_string();
    crate::ffi_out::console_error(&msg);

    core::arch::wasm32::unreachable()
}
