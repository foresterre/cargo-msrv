use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::ReleaseSource;

pub const VALUES: CliValues<ReleaseSource> = CliValues::new(&[
    CliValue::new("rust-changelog", ReleaseSource::RustChangelog),
    #[cfg(feature = "rust-releases-dist-source")]
    CliValue::new("rust-dist", ReleaseSource::RustDist),
]);
