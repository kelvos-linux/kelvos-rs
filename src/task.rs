#[cfg(target_os = "none")]
use crate::syscall;

pub use kelvos_shared_ring::{ExitStatus, Pid, Tid, BOOT_CHILD_IMAGE};

/// Legacy name for the process identifier returned by spawn.
pub type TaskId = Pid;

/// Reasons a task request could not be completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The process is not running on the freestanding Kelvos target.
    UnsupportedPlatform,
    /// The kernel returned an invalid task identifier.
    Protocol,
    /// The kernel returned a negative error result.
    Kernel(i64),
}

/// Starts the child executable from the boot image registry.
pub fn spawn() -> Result<Pid, Error> {
    spawn_image(BOOT_CHILD_IMAGE, 0)
}

/// Starts a registered executable and passes one scalar argument to its entry point.
pub fn spawn_image(image: u64, argument: u64) -> Result<Pid, Error> {
    Pid::from_raw(identifier(kelvos_shared_ring::SYS_SPAWN, image, argument)?)
        .ok_or(Error::Protocol)
}

/// Returns the current process identifier.
pub fn current_pid() -> Result<Pid, Error> {
    Pid::from_raw(identifier(kelvos_shared_ring::SYS_GET_PID, 0, 0)?).ok_or(Error::Protocol)
}

/// Returns the current thread identifier.
pub fn current_tid() -> Result<Tid, Error> {
    Tid::from_raw(identifier(kelvos_shared_ring::SYS_GET_TID, 0, 0)?).ok_or(Error::Protocol)
}

fn identifier(number: u64, arg: u64, extra: u64) -> Result<u32, Error> {
    u32::try_from(request(number, arg, extra)?).map_err(|_| Error::Protocol)
}

fn request(_number: u64, _arg: u64, _extra: u64) -> Result<u64, Error> {
    #[cfg(not(target_os = "none"))]
    return Err(Error::UnsupportedPlatform);

    #[cfg(target_os = "none")]
    {
        let result = syscall::call(_number, _arg, _extra);
        u64::try_from(result).map_err(|_| Error::Kernel(result))
    }
}

/// Waits for a direct child and collects its termination status once.
pub fn wait_child(pid: Pid) -> Result<ExitStatus, Error> {
    let value = request(kelvos_shared_ring::SYS_WAIT_CHILD, u64::from(pid.get()), 0)?;
    ExitStatus::decode(value).ok_or(Error::Protocol)
}

/// Collects the child's status if available, returning immediately otherwise.
pub fn try_wait_child(pid: Pid) -> Result<Option<ExitStatus>, Error> {
    match request(
        kelvos_shared_ring::SYS_WAIT_CHILD,
        u64::from(pid.get()),
        kelvos_shared_ring::WAIT_NO_HANG,
    ) {
        Err(Error::Kernel(-11)) => Ok(None),
        result => ExitStatus::decode(result?).map(Some).ok_or(Error::Protocol),
    }
}

/// Terminates the current process with status zero.
#[cfg(target_os = "none")]
pub fn exit() -> ! {
    exit_with(0)
}

/// Terminates the current process with the supplied status.
#[cfg(target_os = "none")]
pub fn exit_with(code: u32) -> ! {
    let _ = syscall::call(kelvos_shared_ring::SYS_EXIT, u64::from(code), 0);
    panic!("exit syscall returned")
}

/// Spins indefinitely without issuing a scheduler or wait syscall.
///
/// This is useful for minimal freestanding applications that have no work to do.
pub fn idle_forever() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
