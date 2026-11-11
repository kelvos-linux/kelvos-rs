use core::alloc::Layout;
use core::mem::size_of;
use core::ptr::NonNull;

use kelvos_shared_ring::SharedRing;
use kelvos_shared_ring::{
    MAX_SPAWNED_TASKS, SPAWN_STACK_TOP, USER_IMAGE_ADDRESS, USER_IMAGE_CAPACITY, USER_RING_ADDRESS,
    USER_STACK_SIZE, USER_STACK_TOP,
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
    /// Shared submission/completion ring.
    pub shared_ring: Region,
    /// Initial task stack.
    pub initial_stack: Region,
    /// Contiguous range reserved for spawned task stacks.
    pub spawned_stacks: Region,
}

/// Describes the fixed userspace mappings supported by the current kernel.
pub const fn user_layout() -> UserLayout {
    let user_image_address = USER_IMAGE_ADDRESS as usize;
    let ring_address = USER_RING_ADDRESS as usize;
    let initial_stack_top = USER_STACK_TOP as usize;
    let spawned_stack_top = SPAWN_STACK_TOP as usize;
    let stack_bytes = USER_STACK_SIZE;

    UserLayout {
        image: Region {
            start: user_image_address,
            len: USER_IMAGE_CAPACITY,
        },
        shared_ring: Region {
            start: ring_address,
            len: size_of::<SharedRing>(),
        },
        initial_stack: Region {
            start: initial_stack_top - stack_bytes,
            len: stack_bytes,
        },
        spawned_stacks: Region {
            start: spawned_stack_top - stack_bytes,
            len: stack_bytes * MAX_SPAWNED_TASKS as usize,
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
/// The current Kelvos kernel maps fixed image, ring, and stack regions and does
/// not provide a userspace allocation syscall.
pub fn allocate(_layout: Layout) -> Result<NonNull<u8>, Error> {
    Err(Error::Unsupported)
}

#[cfg(target_os = "none")]
pub(crate) unsafe fn shared_ring() -> &'static SharedRing {
    unsafe { &*(USER_RING_ADDRESS as *const SharedRing) }
}

#[cfg(target_os = "none")]
pub(crate) const fn spawn_entry_address() -> u64 {
    USER_IMAGE_ADDRESS
}

#[cfg(target_os = "none")]
pub(crate) const fn spawn_stack_top() -> u64 {
    SPAWN_STACK_TOP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_current_fixed_mapping_layout() {
        let layout = user_layout();
        assert_eq!(layout.image.start, USER_IMAGE_ADDRESS as usize);
        assert_eq!(layout.image.len, USER_IMAGE_CAPACITY);
        assert_eq!(layout.shared_ring.start, USER_RING_ADDRESS as usize);
        assert_eq!(layout.shared_ring.len, size_of::<SharedRing>());
        assert_eq!(
            layout.initial_stack.start + layout.initial_stack.len,
            USER_STACK_TOP as usize
        );
        assert_eq!(
            layout.spawned_stacks.start + USER_STACK_SIZE,
            SPAWN_STACK_TOP as usize
        );
        assert_eq!(
            layout.spawned_stacks.len,
            USER_STACK_SIZE * MAX_SPAWNED_TASKS as usize
        );
    }

    #[test]
    fn reports_missing_kernel_allocator() {
        let layout = Layout::from_size_align(16, 8).unwrap();
        assert_eq!(allocate(layout), Err(Error::Unsupported));
    }
}
