use crate::error::FetchIndexError;
use cargo_msrv_context::types::ReleaseSource;
use cargo_msrv_reporter::Reporter;
use cargo_msrv_reporter::event::FetchIndex;
#[cfg(feature = "rust-releases-offline-source")]
use rust_releases::BundledReleases;
use rust_releases::RustChangelog;
#[cfg(feature = "rust-releases-dist-source")]
use rust_releases::RustDist;
use rust_releases::core::{RustRelease, Stable, StableReleases};
use std::iter::FromIterator;

#[derive(Clone, Debug, Default)]
pub struct ReleaseIndex {
    releases: StableReleases,
}

impl ReleaseIndex {
    pub fn stable_releases(&self) -> &StableReleases {
        &self.releases
    }

    pub fn releases(&self) -> Vec<RustRelease<Stable>> {
        let mut releases = self.releases.iter().cloned().collect::<Vec<_>>();
        releases.reverse();

        releases
    }
}

impl From<StableReleases> for ReleaseIndex {
    fn from(releases: StableReleases) -> Self {
        Self { releases }
    }
}

impl FromIterator<RustRelease<Stable>> for ReleaseIndex {
    fn from_iter<T: IntoIterator<Item = RustRelease<Stable>>>(iter: T) -> Self {
        Self {
            releases: iter.into_iter().collect(),
        }
    }
}

pub fn fetch_index(
    reporter: &impl Reporter,
    release_source: ReleaseSource,
) -> Result<ReleaseIndex, FetchIndexError> {
    reporter.run_scoped_event(FetchIndex::new(release_source), || {
        let source: &'static str = release_source.into();
        info!(source = source, "fetching index");

        let releases = match release_source {
            ReleaseSource::RustChangelog => RustChangelog::new_ureq_cached_client()?.fetch()?,
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => RustDist::new_aws_cached_client()?.stable().fetch()?,
            #[cfg(feature = "rust-releases-offline-source")]
            ReleaseSource::Offline => {
                let bundle = BundledReleases::new();
                info!(generated_on = %bundle.generated_on().ymd(), "using bundled index");

                bundle.stable()
            }
        };

        Ok(ReleaseIndex::from(releases))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releases_are_ordered_from_most_to_least_recent() {
        let index = ReleaseIndex::from_iter(vec![
            RustRelease::new(Stable::new(1, 54, 0), None, []),
            RustRelease::new(Stable::new(1, 56, 0), None, []),
            RustRelease::new(Stable::new(1, 55, 0), None, []),
        ]);

        let versions = index
            .releases()
            .iter()
            .map(|release| release.version().clone())
            .collect::<Vec<_>>();

        assert_eq!(
            versions,
            vec![
                Stable::new(1, 56, 0),
                Stable::new(1, 55, 0),
                Stable::new(1, 54, 0)
            ]
        );
    }
}
