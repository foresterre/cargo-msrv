use crate::error::FetchIndexError;
use cargo_msrv_context::RustReleasesContext;
use cargo_msrv_context::types::ReleaseSource;
use cargo_msrv_reporter::Reporter;
use cargo_msrv_reporter::event::FetchIndex;
use rust_releases::BundledReleases;
#[cfg(feature = "rust-releases-github-source")]
use rust_releases::GithubReleases;
#[cfg(feature = "rust-releases-changelog-source")]
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
    rust_releases: &RustReleasesContext,
) -> Result<ReleaseIndex, FetchIndexError> {
    let release_source = rust_releases.release_source;

    reporter.run_scoped_event(FetchIndex::new(release_source), || {
        let source: &'static str = release_source.into();
        info!(source = source, "fetching index");

        let releases = fetch_releases(release_source, rust_releases)?;

        Ok(ReleaseIndex::from(releases))
    })
}

fn fetch_releases(
    release_source: ReleaseSource,
    #[cfg_attr(
        not(any(
            feature = "rust-releases-changelog-source",
            feature = "rust-releases-github-source",
            feature = "rust-releases-dist-source"
        )),
        expect(unused_variables)
    )]
    rust_releases: &RustReleasesContext,
) -> Result<StableReleases, FetchIndexError> {
    let releases = match release_source {
        #[cfg(feature = "rust-releases-changelog-source")]
        ReleaseSource::RustChangelog => RustChangelog::new_ureq_cached_client()?.fetch()?,
        #[cfg(feature = "rust-releases-github-source")]
        ReleaseSource::GitHub => GithubReleases::new_ureq_cached_client()?.fetch()?,
        #[cfg(feature = "rust-releases-dist-source")]
        ReleaseSource::RustDist => RustDist::new_aws_cached_client()?.stable().fetch()?,
        ReleaseSource::Offline => {
            let bundle = BundledReleases::new();
            info!(generated_on = %bundle.generated_on().ymd(), "using bundled index");

            bundle.stable()
        }
        #[cfg(any(
            feature = "rust-releases-changelog-source",
            feature = "rust-releases-github-source",
            feature = "rust-releases-dist-source"
        ))]
        ReleaseSource::OfflineUnlessOutdated => bundled::fetch_unless_outdated(
            BundledReleases::new(),
            time::OffsetDateTime::now_utc().date(),
            rust_releases,
        )?,
    };

    Ok(releases)
}

#[cfg(any(
    feature = "rust-releases-changelog-source",
    feature = "rust-releases-github-source",
    feature = "rust-releases-dist-source"
))]
mod bundled {
    use super::fetch_releases;
    use crate::error::FetchIndexError;
    use cargo_msrv_context::RustReleasesContext;
    use cargo_msrv_context::types::{BundledMaxAge, ReleaseSource};
    use rust_releases::BundledReleases;
    use rust_releases::core::StableReleases;
    use rust_releases::core::merge::builder::MergeBuilder;
    use rust_releases::core::merge::strategy::release_date::PreferLeftDate;
    use rust_releases::core::merge::strategy::toolchains::PreferLeftToolchains;
    use rust_releases::core::rust_release::date::Date;

    pub(super) fn fetch_unless_outdated(
        bundle: BundledReleases,
        today: time::Date,
        rust_releases: &RustReleasesContext,
    ) -> Result<StableReleases, FetchIndexError> {
        let fallback = rust_releases.bundled_fallback;
        let generated_on = bundle.generated_on();

        match age_in_days(&generated_on, today) {
            Some(age) if !is_outdated(age, fallback.max_age) => {
                info!(
                    generated_on = %generated_on.ymd(),
                    age_in_days = age,
                    max_age_in_days = fallback.max_age.days(),
                    "using bundled index"
                );

                return Ok(bundle.stable());
            }
            Some(age) => {
                info!(
                    generated_on = %generated_on.ymd(),
                    age_in_days = age,
                    max_age_in_days = fallback.max_age.days(),
                    "bundled index is outdated"
                );
            }
            None => {
                warn!(
                    generated_on = %generated_on.ymd(),
                    "unable to tell the age of the bundled index, treating it as outdated"
                );
            }
        }

        let fallback_source = ReleaseSource::from(fallback.source);
        let source: &'static str = fallback_source.into();
        info!(source = source, "fetching fallback index");

        let releases = fetch_releases(fallback_source, rust_releases)?;
        let preferred = merge_preferring_bundled(bundle.stable(), releases);

        Ok(preferred)
    }

    fn age_in_days(generated_on: &Date, today: time::Date) -> Option<i64> {
        let month = time::Month::try_from(generated_on.month()).ok()?;
        let generated_on = time::Date::from_calendar_date(
            i32::from(generated_on.year()),
            month,
            generated_on.day(),
        )
        .ok()?;

        Some((today - generated_on).whole_days())
    }

    fn is_outdated(age_in_days: i64, max_age: BundledMaxAge) -> bool {
        age_in_days > i64::from(max_age.days())
    }

    fn merge_preferring_bundled(
        bundled: StableReleases,
        fetched: StableReleases,
    ) -> StableReleases {
        bundled.merge_with(fetched, |bundled, fetched| {
            MergeBuilder::new(bundled, fetched)
                .date_merge(PreferLeftDate)
                .toolchains_merge(PreferLeftToolchains)
                .finish()
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rust_releases::core::rust_release::toolchain::{
            Channel, Component, Target, TargetSet, Toolchain,
        };
        use rust_releases::core::{RustRelease, Stable};

        fn today(year: i32, month: time::Month, day: u8) -> time::Date {
            time::Date::from_calendar_date(year, month, day).unwrap()
        }

        #[yare::parameterized(
            same_day = { Date::new(2026, 9, 22), today(2026, time::Month::September, 22), 0 },
            next_day = { Date::new(2026, 9, 22), today(2026, time::Month::September, 23), 1 },
            across_leap_day = { Date::new(2024, 2, 28), today(2024, time::Month::March, 1), 2 },
            across_years = { Date::new(2025, 12, 31), today(2026, time::Month::January, 1), 1 },
            clock_behind = { Date::new(2026, 9, 22), today(2026, time::Month::September, 20), -2 },
        )]
        fn age_of_bundle(generated_on: Date, today: time::Date, expected: i64) {
            assert_eq!(age_in_days(&generated_on, today), Some(expected));
        }

        #[yare::parameterized(
            month_zero = { Date::new(2026, 0, 1) },
            month_thirteen = { Date::new(2026, 13, 1) },
            day_zero = { Date::new(2026, 1, 0) },
            not_a_leap_year = { Date::new(2025, 2, 29) },
        )]
        fn age_of_invalid_bundle_date_is_unknown(generated_on: Date) {
            let today = today(2026, time::Month::September, 22);

            assert_eq!(age_in_days(&generated_on, today), None);
        }

        #[yare::parameterized(
            clock_behind = { -1, 42, false },
            fresh = { 0, 42, false },
            at_max_age = { 42, 42, false },
            past_max_age = { 43, 42, true },
            zero_max_age = { 1, 0, true },
        )]
        fn outdated(age_in_days: i64, max_age_in_days: u32, expected: bool) {
            let max_age = BundledMaxAge::from_days(max_age_in_days);

            assert_eq!(is_outdated(age_in_days, max_age), expected);
        }

        fn toolchain(version: Stable, host: &str) -> Toolchain {
            Toolchain::new(
                Channel::Stable(version),
                None,
                Target::try_from_target_triple(host).unwrap(),
                [Component::new("rustc")].into_iter().collect(),
                TargetSet::default(),
            )
        }

        #[test]
        fn merge_prefers_bundled_releases() {
            let version = Stable::new(1, 80, 0);
            let bundled_toolchain = toolchain(version.clone(), "x86_64-unknown-linux-gnu");
            let fetched_toolchain = toolchain(version.clone(), "aarch64-apple-darwin");

            let bundled = StableReleases::new([RustRelease::new(
                version.clone(),
                Some(Date::new(2024, 7, 25)),
                [bundled_toolchain.clone()],
            )]);
            let fetched = StableReleases::new([RustRelease::new(
                version.clone(),
                Some(Date::new(2000, 1, 1)),
                [fetched_toolchain],
            )]);

            let merged = merge_preferring_bundled(bundled, fetched);
            let releases = merged.iter().collect::<Vec<_>>();

            assert_eq!(releases.len(), 1);
            assert_eq!(releases[0].version(), &version);
            assert_eq!(releases[0].release_date(), Some(&Date::new(2024, 7, 25)));
            assert_eq!(
                releases[0].toolchains_iter().collect::<Vec<_>>(),
                vec![&bundled_toolchain]
            );
        }

        #[test]
        fn merge_takes_fetched_date_when_bundled_has_none() {
            let version = Stable::new(1, 80, 0);

            let bundled = StableReleases::new([RustRelease::new(version.clone(), None, [])]);
            let fetched =
                StableReleases::new([RustRelease::new(version, Some(Date::new(2024, 7, 25)), [])]);

            let merged = merge_preferring_bundled(bundled, fetched);
            let release = merged.iter().next().unwrap();

            assert_eq!(release.release_date(), Some(&Date::new(2024, 7, 25)));
        }

        #[test]
        fn merge_keeps_releases_from_both_sources() {
            let bundled = StableReleases::new([
                RustRelease::new(Stable::new(1, 79, 0), None, []),
                RustRelease::new(Stable::new(1, 80, 0), None, []),
            ]);
            let fetched = StableReleases::new([
                RustRelease::new(Stable::new(1, 80, 0), None, []),
                RustRelease::new(Stable::new(1, 81, 0), None, []),
                RustRelease::new(Stable::new(1, 82, 0), None, []),
            ]);

            let merged = merge_preferring_bundled(bundled, fetched);
            let versions = merged
                .iter()
                .map(|release| release.version().clone())
                .collect::<Vec<_>>();

            assert_eq!(
                versions,
                vec![
                    Stable::new(1, 79, 0),
                    Stable::new(1, 80, 0),
                    Stable::new(1, 81, 0),
                    Stable::new(1, 82, 0),
                ]
            );
        }

        #[test]
        fn fresh_bundle_is_used_without_fetching() {
            let bundle = BundledReleases::new();
            let generated_on = bundle.generated_on();
            let today = time::Date::from_calendar_date(
                i32::from(generated_on.year()),
                time::Month::try_from(generated_on.month()).unwrap(),
                generated_on.day(),
            )
            .unwrap();

            let releases =
                fetch_unless_outdated(bundle, today, &RustReleasesContext::default()).unwrap();

            assert_eq!(releases, bundle.stable());
        }
    }
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
