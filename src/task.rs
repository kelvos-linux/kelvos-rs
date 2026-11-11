#[cfg(target_os = "none")]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "none")]
use kelvos_shared_ring::{Completion, Submission, OP_SPAWN};

#[cfg(target_os = "none")]
use crate::{memory, syscall};

#[cfg(target_os = "none")]
static SPAWN_LOCK: AtomicBool = AtomicBool::new(false);

/// Identifies a task created by the kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskId(u32);

impl TaskId {
    /// Returns the kernel-assigned task identifier.
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Reasons a task request could not be completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The process is not running on the freestanding Kelvos target.
    UnsupportedPlatform,
    /// Another caller is currently submitting a task request.
    Busy,
    /// The shared submission queue has no free entries.
    QueueFull,
    /// The kernel did not publish a completion for the request.
    NoCompletion,
    /// The kernel completion did not match the submitted request.
    Protocol,
    /// The kernel returned a negative error result.
    Kernel(i64),
}

#[cfg(target_os = "none")]
struct SpawnGuard;

#[cfg(target_os = "none")]
impl SpawnGuard {
    fn acquire() -> Result<Self, Error> {
        SPAWN_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| Self)
            .map_err(|_| Error::Busy)
    }
}

#[cfg(target_os = "none")]
impl Drop for SpawnGuard {
    fn drop(&mut self) {
        SPAWN_LOCK.store(false, Ordering::Release);
    }
}

/// Creates a task using the child image and stack currently supported by the kernel.
///
/// The request is serialized with other callers because the kernel ABI exposes a
/// single-producer/single-consumer shared ring.
pub fn spawn() -> Result<TaskId, Error> {
    #[cfg(not(target_os = "none"))]
    return Err(Error::UnsupportedPlatform);

    #[cfg(target_os = "none")]
    {
        let _guard = SpawnGuard::acquire()?;
        // The kernel maps this shared ring before entering userspace.
        let ring = unsafe { memory::shared_ring() };
        let tail = ring.sq_tail.load(Ordering::Relaxed);

        if !ring.producer_has_space(tail) {
            return Err(Error::QueueFull);
        }

        unsafe {
            ring.write_submission(
                tail,
                Submission {
                    opcode: OP_SPAWN,
                    flags: 0,
                    entry_point: memory::spawn_entry_address(),
                    stack_top: memory::spawn_stack_top(),
                },
            );
        }
        ring.sq_tail.store(tail.wrapping_add(1), Ordering::Release);

        syscall::notify_ring();

        let head = ring.cq_head.load(Ordering::Relaxed);
        if !ring.consumer_has_entries(head) {
            return Err(Error::NoCompletion);
        }

        let completion: Completion = unsafe { ring.read_completion(head) };
        ring.cq_head.store(head.wrapping_add(1), Ordering::Release);

        if completion.user_data != u64::from(tail) {
            return Err(Error::Protocol);
        }
        if completion.result < 0 {
            return Err(Error::Kernel(completion.result));
        }
        if completion.result == 0 {
            return Err(Error::Protocol);
        }

        u32::try_from(completion.result)
            .map(TaskId)
            .map_err(|_| Error::Protocol)
    }
}

/// Spins indefinitely without issuing a scheduler or wait syscall.
///
/// This is useful for minimal freestanding applications that have no work to do.
pub fn idle_forever() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_id_exposes_kernel_identifier() {
        assert_eq!(TaskId(7).get(), 7);
    }

    #[cfg(not(target_os = "none"))]
    #[test]
    fn spawn_is_unavailable_outside_freestanding_targets() {
        assert_eq!(spawn(), Err(Error::UnsupportedPlatform));
    }
}
