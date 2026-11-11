/// Console operations currently supported by the Kelvos userspace ABI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The current kernel ABI has no userspace console-output operation.
    Unsupported,
}

/// Writes a string to the kernel console.
///
/// The current Kelvos kernel does not expose console output to userspace, so
/// this returns [`Error::Unsupported`] without performing I/O.
pub fn write_str(_text: &str) -> Result<(), Error> {
    Err(Error::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_kernel_console_support() {
        assert_eq!(write_str("test"), Err(Error::Unsupported));
    }
}
