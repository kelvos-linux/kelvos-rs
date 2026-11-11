use proc_macro::TokenStream;

/// Generates the freestanding Kelvos `_start` symbol for an application entry point.
#[proc_macro_attribute]
pub fn entry(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let mut output = item;
    output.extend(
        r#"
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn __kelvos_rs_start() -> ! {
            ::kelvos_rs::__initialize_runtime();
            main()
        }

        ::core::arch::global_asm!(
            ".section .text._start, \"ax\"",
            ".global _start",
            ".type _start,@function",
            "_start:",
            "call __kelvos_rs_start",
            "1:",
            "pause",
            "jmp 1b",
        );
        "#
        .parse::<TokenStream>()
        .expect("generated Kelvos entry point should be valid Rust tokens"),
    );
    output
}
