#[derive(Debug, thiserror::Error)]
pub enum FetchIndexError {
    #[error(transparent)]
    RustReleasesSource(
        #[from] rust_releases::RustChangelogError<rust_releases::HttpCachedClientError>,
    ),

    #[error(transparent)]
    RustReleasesCacheDir(#[from] rust_releases::BaseCacheDirError),

    #[error(transparent)]
    #[cfg(feature = "rust-releases-dist-source")]
    RustReleasesRustDistSource(
        #[from] rust_releases::rust_dist::CachedDistError<rust_releases::rust_dist::AwsError>,
    ),

    #[error(transparent)]
    #[cfg(feature = "rust-releases-dist-source")]
    RustReleasesRustDistSetup(#[from] rust_releases::rust_dist::AwsSetupError),

    #[error("Unable to print event output")]
    Storyteller,
}

impl<T> From<storyteller::EventReporterError<T>> for FetchIndexError {
    fn from(_: storyteller::EventReporterError<T>) -> Self {
        Self::Storyteller
    }
}
