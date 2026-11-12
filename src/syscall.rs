use core::arch::asm;

pub(crate) fn call(number: u64, arg: u64, extra: u64) -> i64 {
    let result: u64;
    unsafe {
        asm!("int 0x80", inlateout("rax") number => result, in("rdi") arg, in("rsi") extra, options(nostack));
    }
    result as i64
}
