use core::arch::asm;

const RING_NOTIFY: u64 = 1;

pub(crate) fn notify_ring() {
    unsafe {
        asm!("int 0x80", in("rax") RING_NOTIFY, options(nostack));
    }
}
