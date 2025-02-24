use std::fmt;

/// A type to represent the Minimal Supported Rust Version, also known as the
/// MSRV.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MSRV {
    /// A toolchain is compatible, if the outcome of a toolchain check results in a success
    version: rust_toolchain::RustVersion,
}

impl MSRV {
    pub fn new(msrv: rust_toolchain::RustVersion) -> Self {
        Self { version: msrv }
    }

    pub fn msrv(&self) -> rust_toolchain::RustVersion {
        self.version
    }

    pub fn version(&self) -> impl fmt::Display {
        self.version
    }

    pub fn short_version(&self) -> impl fmt::Display {
        version_number::BaseVersion::new(self.version.major(), self.version.minor())
    }
}

impl fmt::Display for MSRV {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", &self.version))
    }
}

#[cfg(test)]
mod tests {
    use crate::msrv::MSRV;

    #[test]
    fn msrv_method() {
        let version = rust_toolchain::RustVersion::new(1, 2, 3);
        let msrv = MSRV::new(version);

        assert_eq!(msrv.msrv(), version);
    }

    #[test]
    fn version_str() {
        let version = rust_toolchain::RustVersion::new(1, 2, 3);
        let msrv = MSRV::new(version);

        assert_eq!(msrv.version().to_string(), "1.2.3".to_string());
    }

    #[test]
    fn short_version_str() {
        let version = rust_toolchain::RustVersion::new(1, 2, 3);
        let msrv = MSRV::new(version);

        assert_eq!(msrv.short_version().to_string(), "1.2".to_string());
    }
}
