#![no_std]

#[cfg(not(target_arch = "x86_64"))]
compile_error!("kelvos-rs currently supports the x86_64 Kelvos ABI only");

pub mod console;
pub mod memory;
#[cfg(target_os = "none")]
mod syscall;
pub mod task;

pub use kelvos_rs_macros::entry;

/// Items intended for glob-import by freestanding userspace applications.
pub mod prelude {
    pub use crate::entry;
    pub use crate::task;
}

#[doc(hidden)]
pub fn __initialize_runtime() {
    unsafe {
        core::arch::asm!("cld", options(nomem, nostack));
    }
}
