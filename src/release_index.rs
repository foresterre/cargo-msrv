use crate::config::{Config, ReleaseSource};
use crate::error::TResult;
use crate::reporter::event::FetchIndex;
use crate::reporter::Reporter;
#[cfg(feature = "rust-releases-dist-source")]
use rust_releases::RustDist;
use rust_releases::{Channel, FetchResources, ReleaseIndex, RustChangelog, Source};

pub fn fetch_index(config: &Config, reporter: &impl Reporter) -> TResult<ReleaseIndex> {
    reporter.run_scoped_event(FetchIndex::new(config.release_source()), || {
        let source = config.release_source();

        info!(
            source = Into::<&'static str>::into(source),
            "fetching index"
        );

        let index = match config.release_source() {
            ReleaseSource::RustChangelog => {
                RustChangelog::fetch_channel(Channel::Stable)?.build_index()?
            }
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => RustDist::fetch_channel(Channel::Stable)?.build_index()?,
        };

        Ok(index)
    })
}
