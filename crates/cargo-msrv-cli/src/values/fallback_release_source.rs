use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::FallbackReleaseSource;

pub const VALUES: CliValues<FallbackReleaseSource> = CliValues::new(&[
    CliValue::new("rust-changelog", FallbackReleaseSource::RustChangelog),
    #[cfg(feature = "rust-releases-github-source")]
    CliValue::new("github", FallbackReleaseSource::GitHub),
    #[cfg(feature = "rust-releases-dist-source")]
    CliValue::new("rust-dist", FallbackReleaseSource::RustDist),
]);
