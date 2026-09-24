use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::ReleaseSource;

pub const VALUES: CliValues<ReleaseSource> = CliValues::new(&[
    #[cfg(feature = "rust-releases-changelog-source")]
    CliValue::new("rust-changelog", ReleaseSource::RustChangelog),
    #[cfg(feature = "rust-releases-github-source")]
    CliValue::new("github", ReleaseSource::GitHub),
    #[cfg(feature = "rust-releases-dist-source")]
    CliValue::new("rust-dist", ReleaseSource::RustDist),
    CliValue::new("bundled", ReleaseSource::Bundled),
    #[cfg(any(
        feature = "rust-releases-changelog-source",
        feature = "rust-releases-github-source",
        feature = "rust-releases-dist-source"
    ))]
    CliValue::new(
        "bundled-unless-outdated",
        ReleaseSource::BundledUnlessOutdated,
    ),
]);
