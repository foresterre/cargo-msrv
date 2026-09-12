use crate::release::to_semver;
use rust_releases::core::rust_release::toolchain::{ComponentSet, Target};
use rust_releases::core::{RustRelease, Stable};

#[derive(Clone, Debug, PartialEq)]
pub enum ToolchainAvailability {
    Available,
    Unavailable(ToolchainUnavailable),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ToolchainUnavailable {
    #[error("no toolchain for host '{host}'")]
    Host { host: String },

    #[error("target '{target}' unavailable")]
    Target { target: String },

    #[error("component(s) '{}' unavailable", missing.join(", "))]
    Components { missing: Vec<String> },
}

pub struct AvailabilityFilter<'ctx> {
    host: &'ctx str,
    target: &'ctx str,
    components: &'ctx [&'ctx str],
}

impl<'ctx> AvailabilityFilter<'ctx> {
    pub fn new(host: &'ctx str, target: &'ctx str, components: &'ctx [&'ctx str]) -> Self {
        Self {
            host,
            target,
            components,
        }
    }

    pub fn availability(&self, release: &RustRelease<Stable>) -> ToolchainAvailability {
        if release.toolchains.is_empty() {
            return ToolchainAvailability::Unknown;
        }

        let (Ok(host), Ok(target)) = (
            Target::try_from_target_triple(self.host),
            Target::try_from_target_triple(self.target),
        ) else {
            return ToolchainAvailability::Unknown;
        };

        let Some(toolchain) = release
            .toolchains
            .iter()
            .find(|toolchain| toolchain.host() == &host)
        else {
            return ToolchainAvailability::Unavailable(ToolchainUnavailable::Host {
                host: self.host.to_string(),
            });
        };

        let targets = toolchain.targets();

        if !targets.is_empty() && !targets.contains(&target) {
            return ToolchainAvailability::Unavailable(ToolchainUnavailable::Target {
                target: self.target.to_string(),
            });
        }

        let components = toolchain.components();

        if !components.is_empty() {
            let missing = self
                .components
                .iter()
                .filter(|requested| !provides_component(components, requested))
                .map(|requested| requested.to_string())
                .collect::<Vec<_>>();

            if !missing.is_empty() {
                return ToolchainAvailability::Unavailable(ToolchainUnavailable::Components {
                    missing,
                });
            }
        }

        ToolchainAvailability::Available
    }

    pub fn filter(&self, releases: &[RustRelease<Stable>]) -> AvailabilityOutcome {
        let mut included = Vec::with_capacity(releases.len());
        let mut excluded = Vec::new();

        for release in releases {
            match self.availability(release) {
                ToolchainAvailability::Available | ToolchainAvailability::Unknown => {
                    included.push(release.clone())
                }
                ToolchainAvailability::Unavailable(reason) => {
                    let version = to_semver(release.version());

                    info!(%version, %reason, "excluded release from search space");

                    excluded.push(ExcludedRelease { version, reason });
                }
            }
        }

        AvailabilityOutcome { included, excluded }
    }
}

#[derive(Debug)]
pub struct AvailabilityOutcome {
    included: Vec<RustRelease<Stable>>,
    excluded: Vec<ExcludedRelease>,
}

impl AvailabilityOutcome {
    pub fn included(&self) -> &[RustRelease<Stable>] {
        &self.included
    }

    pub fn excluded(&self) -> &[ExcludedRelease] {
        &self.excluded
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExcludedRelease {
    version: semver::Version,
    reason: ToolchainUnavailable,
}

impl ExcludedRelease {
    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    pub fn reason(&self) -> &ToolchainUnavailable {
        &self.reason
    }
}

fn provides_component(components: &ComponentSet, requested: &str) -> bool {
    components.iter().any(|component| {
        let name = component.name();

        name == requested || name.strip_suffix("-preview") == Some(requested)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_releases::core::rust_release::toolchain::{Channel, Component, Toolchain};

    fn toolchain(host: &str, targets: &[&str], components: &[&str]) -> Toolchain {
        Toolchain::new(
            Channel::Stable(Stable::new(1, 2, 3)),
            None,
            Target::try_from_target_triple(host).unwrap(),
            components
                .iter()
                .map(|name| Component::new(name.to_string()))
                .collect(),
            targets
                .iter()
                .map(|triple| Target::try_from_target_triple(triple).unwrap())
                .collect(),
        )
    }

    fn release(toolchains: impl IntoIterator<Item = Toolchain>) -> RustRelease<Stable> {
        RustRelease::new(Stable::new(1, 2, 3), None, toolchains)
    }

    const HOST: &str = "x86_64-unknown-linux-gnu";
    const OTHER: &str = "aarch64-apple-darwin";

    #[test]
    fn release_without_toolchains_is_unknown() {
        let filter = AvailabilityFilter::new(HOST, HOST, &[]);

        assert_eq!(
            filter.availability(&release([])),
            ToolchainAvailability::Unknown
        );
    }

    #[test]
    fn unparseable_triple_is_unknown() {
        let filter = AvailabilityFilter::new("x", "x", &[]);
        let release = release([toolchain(HOST, &[HOST], &["cargo"])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Unknown
        );
    }

    #[test]
    fn host_toolchain_which_provides_the_target() {
        let filter = AvailabilityFilter::new(HOST, OTHER, &[]);
        let release = release([toolchain(HOST, &[HOST, OTHER], &[])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Available
        );
    }

    #[test]
    fn host_toolchain_is_its_own_target() {
        let filter = AvailabilityFilter::new(HOST, HOST, &[]);
        let release = release([toolchain(HOST, &[HOST], &[])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Available
        );
    }

    #[test]
    fn no_toolchain_for_host() {
        let filter = AvailabilityFilter::new(OTHER, OTHER, &[]);
        let release = release([toolchain(HOST, &[HOST, OTHER], &[])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Unavailable(ToolchainUnavailable::Host {
                host: OTHER.to_string()
            })
        );
    }

    #[test]
    fn target_not_provided_by_host_toolchain() {
        let filter = AvailabilityFilter::new(HOST, OTHER, &[]);
        let release = release([toolchain(HOST, &[HOST], &[])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Unavailable(ToolchainUnavailable::Target {
                target: OTHER.to_string()
            })
        );
    }

    #[test]
    fn toolchain_without_targets_does_not_reject_the_target() {
        let filter = AvailabilityFilter::new(HOST, OTHER, &[]);
        let release = release([toolchain(HOST, &[], &[])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Available
        );
    }

    #[test]
    fn requested_components_are_provided() {
        let filter = AvailabilityFilter::new(HOST, HOST, &["cargo", "rustc"]);
        let release = release([toolchain(HOST, &[HOST], &["cargo", "rustc", "rust-std"])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Available
        );
    }

    #[test]
    fn a_preview_component_is_provided_by_its_shorthand() {
        let filter = AvailabilityFilter::new(HOST, HOST, &["clippy", "miri"]);
        let release = release([toolchain(
            HOST,
            &[HOST],
            &["clippy-preview", "miri-preview"],
        )]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Available
        );
    }

    #[test]
    fn requested_components_are_not_provided() {
        let filter = AvailabilityFilter::new(HOST, HOST, &["cargo", "miri", "rust-analyzer"]);
        let release = release([toolchain(HOST, &[HOST], &["cargo", "rustc"])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Unavailable(ToolchainUnavailable::Components {
                missing: vec!["miri".to_string(), "rust-analyzer".to_string()]
            })
        );
    }

    #[test]
    fn toolchain_without_components_does_not_reject_a_component() {
        let filter = AvailabilityFilter::new(HOST, HOST, &["miri"]);
        let release = release([toolchain(HOST, &[HOST], &[])]);

        assert_eq!(
            filter.availability(&release),
            ToolchainAvailability::Available
        );
    }

    #[test]
    fn filter_keeps_available_and_unknown_releases() {
        let filter = AvailabilityFilter::new(HOST, HOST, &[]);

        let available =
            RustRelease::new(Stable::new(1, 60, 0), None, [toolchain(HOST, &[HOST], &[])]);
        let unknown = RustRelease::new(Stable::new(1, 59, 0), None, []);
        let unavailable = RustRelease::new(
            Stable::new(1, 58, 0),
            None,
            [toolchain(OTHER, &[OTHER], &[])],
        );

        let outcome = filter.filter(&[available, unknown, unavailable]);

        let included = outcome
            .included()
            .iter()
            .map(|release| to_semver(release.version()))
            .collect::<Vec<_>>();

        assert_eq!(
            included,
            vec![
                semver::Version::new(1, 60, 0),
                semver::Version::new(1, 59, 0)
            ]
        );

        assert_eq!(
            outcome.excluded(),
            &[ExcludedRelease {
                version: semver::Version::new(1, 58, 0),
                reason: ToolchainUnavailable::Host {
                    host: HOST.to_string()
                }
            }]
        );
    }

    #[cfg(feature = "rust-releases-offline-source")]
    mod bundled {
        use super::*;
        use rust_releases::BundledReleases;

        fn availability(host: &str, version: semver::Version) -> ToolchainAvailability {
            let filter = AvailabilityFilter::new(host, host, &[]);
            let releases = BundledReleases::new().stable();

            let release = releases
                .iter()
                .find(|release| to_semver(release.version()) == version)
                .unwrap_or_else(|| panic!("Rust {} is not bundled", version));

            filter.availability(release)
        }

        #[test]
        fn a_release_without_bundled_toolchains() {
            assert_eq!(
                availability(HOST, semver::Version::new(1, 7, 0)),
                ToolchainAvailability::Unknown
            );
        }

        #[test]
        fn the_first_release_distributed_for_apple_silicon() {
            assert_eq!(
                availability(OTHER, semver::Version::new(1, 49, 0)),
                ToolchainAvailability::Available
            );
        }

        #[test]
        fn the_last_release_not_distributed_for_apple_silicon() {
            assert_eq!(
                availability(OTHER, semver::Version::new(1, 48, 0)),
                ToolchainAvailability::Unavailable(ToolchainUnavailable::Host {
                    host: OTHER.to_string()
                })
            );
        }

        #[test]
        fn a_component_which_is_distributed_as_a_preview() {
            let filter = AvailabilityFilter::new(HOST, HOST, &["clippy"]);
            let releases = BundledReleases::new().stable();

            let release = releases
                .iter()
                .find(|release| to_semver(release.version()) == semver::Version::new(1, 89, 0))
                .unwrap();

            assert_eq!(
                filter.availability(release),
                ToolchainAvailability::Available
            );
        }
    }
}
