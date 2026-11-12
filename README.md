# kelvos-rs

A `#![no_std]` Rust userspace library for the current Kelvos x86_64 kernel ABI.

## Supported API

- `#[kelvos_rs::entry]` generates the `_start` symbol, clears the x86 direction
  flag as required by the Rust calling convention, and calls the application's
  `main() -> !`.
- `task::spawn()` launches the boot child executable and returns its `Pid`.
  PID 1 can also select the image and pass a scalar with `task::spawn_image()`.
- `task::current_pid()` and `task::current_tid()` return the current process
  and thread identifiers.
- `task::wait_child()` collects a direct child’s exit or fault status.
  `task::try_wait_child()` returns immediately when the result is pending.
- `task::exit()` and `task::exit_with(code)` terminate the current process.
- `task::idle_forever()` provides a freestanding spin loop.
- `memory::user_layout()` describes the image and private stack mappings
  provided by the current loader.

The kernel currently provides no userspace console-output or memory-allocation
syscalls. Accordingly, `console::write_str` and `memory::allocate` return
`Unsupported`; they do not issue undocumented syscalls or access privileged
devices.

The syscall number, interrupt instruction, and virtual addresses are kept
inside the library implementation and its shared ABI dependency.

## Build and test

```sh
cargo test
cargo check
```

The `kelv-runtime` source uses this crate through a local path dependency.
Applications should use `#![no_std]`, `#![no_main]`, and mark their
`fn main() -> !` with `#[kelvos_rs::entry]`; they remain responsible for
providing a panic handler.
