use crate::context::{
    CustomCheckContext, DebugOutputContext, EnvironmentContext, RustReleasesContext,
    ToolchainContext, UserOutputContext,
};
use crate::manifest::bare_version::BareVersion;

pub struct VerifyContext {
    /// The resolved Rust version, to check against for toolchain compatibility.
    pub rust_version: BareVersion,

    /// The context for Rust releases
    pub rust_releases: RustReleasesContext,

    /// The context for Rust toolchains
    pub toolchain: ToolchainContext,

    /// The context for custom checks to be used with rustup
    pub custom_check: CustomCheckContext,

    /// Resolved environment options
    pub environment: EnvironmentContext,

    /// User output options
    pub user_output: UserOutputContext,

    /// Debug output options
    pub debug_output: DebugOutputContext,
}
