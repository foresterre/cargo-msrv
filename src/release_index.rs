use crate::config::ReleaseSource;
use crate::error::TResult;
use crate::reporter::event::FetchIndex;
use crate::reporter::Reporter;
#[cfg(feature = "rust-releases-dist-source")]
use rust_releases::RustDist;
use rust_releases::{Channel, FetchResources, ReleaseIndex, RustChangelog, Source};

pub fn fetch_index(
    reporter: &impl Reporter,
    release_source: ReleaseSource,
) -> TResult<ReleaseIndex> {
    reporter.run_scoped_event(FetchIndex::new(release_source), || {
        info!(
            source = Into::<&'static str>::into(source),
            "fetching index"
        );

        let index = match release_source {
            ReleaseSource::RustChangelog => {
                RustChangelog::fetch_channel(Channel::Stable)?.build_index()?
            }
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => RustDist::fetch_channel(Channel::Stable)?.build_index()?,
        };

        Ok(index)
    })
}
