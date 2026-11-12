use core::alloc::Layout;
use core::ptr::NonNull;

use kelvos_shared_ring::{
    USER_IMAGE_ADDRESS, USER_IMAGE_CAPACITY, USER_STACK_SIZE, USER_STACK_TOP,
};

/// Describes a virtual memory range in the current userspace layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Region {
    /// Start address of the range.
    pub start: usize,
    /// Length of the range in bytes.
    pub len: usize,
}

/// Memory ranges mapped by the current Kelvos userspace loader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserLayout {
    /// Maximum supported image range; only pages containing the loaded image
    /// are guaranteed to be mapped.
    pub image: Region,
    /// Private stack of the current process.
    pub stack: Region,
}

/// Describes the fixed userspace mappings supported by the current kernel.
pub const fn user_layout() -> UserLayout {
    let user_image_address = USER_IMAGE_ADDRESS as usize;
    let initial_stack_top = USER_STACK_TOP as usize;
    let stack_bytes = USER_STACK_SIZE;

    UserLayout {
        image: Region {
            start: user_image_address,
            len: USER_IMAGE_CAPACITY,
        },
        stack: Region {
            start: initial_stack_top - stack_bytes,
            len: stack_bytes,
        },
    }
}

/// Reasons a memory request could not be completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The current kernel ABI has no userspace memory-allocation operation.
    Unsupported,
}

/// Requests memory from the kernel.
///
/// The current Kelvos kernel maps fixed image and stack regions and does
/// not provide a userspace allocation syscall.
pub fn allocate(_layout: Layout) -> Result<NonNull<u8>, Error> {
    Err(Error::Unsupported)
}
