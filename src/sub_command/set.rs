use crate::context::SetContext;
use crate::context::set::SetTarget;
use crate::error::InvalidMsrvSetError;
use crate::reporter::Reporter;
use crate::reporter::event::{
    AuxiliaryOutput, AuxiliaryOutputItem, Destination, MsrvKind, SetResult,
    UnableToConfirmValidReleaseVersion,
};
use crate::{SubCommand, TResult};
use camino::Utf8Path;
use cargo_msrv_context::{Project, SelectedPackage};
use cargo_msrv_rust_releases::{ReleaseIndex, RustRelease, Stable};
use cargo_msrv_rust_tools::{write_manifest_msrv, write_workspace_msrv};
use version_number::Version;

pub struct Set<'index> {
    release_index: Option<&'index ReleaseIndex>,
}

impl<'index> Set<'index> {
    pub fn new(release_index: Option<&'index ReleaseIndex>) -> Self {
        Self { release_index }
    }
}

impl SubCommand for Set<'_> {
    type Context = SetContext;
    type Output = ();

    fn run(&self, ctx: &Self::Context, reporter: &impl Reporter) -> TResult<Self::Output> {
        let msrv = &ctx.msrv;

        self.check_release(msrv, reporter)?;

        match (&ctx.target, &ctx.environment.project) {
            (SetTarget::WorkspaceRoot(cargo_toml), _) => {
                write_workspace_msrv(cargo_toml, msrv)?;
                report_set_msrv(reporter, cargo_toml, None, msrv)
            }
            (SetTarget::Packages, Project::Cargo(project)) => {
                for package in project.packages() {
                    set_msrv(
                        reporter,
                        &package.manifest_path,
                        Some(SelectedPackage::from(package)),
                        msrv,
                    )?;
                }

                Ok(())
            }
            (SetTarget::Packages, Project::Bare) => {
                set_msrv(reporter, &ctx.environment.manifest(), None, msrv)
            }
        }
    }
}

impl Set<'_> {
    pub(super) fn check_release(&self, msrv: &Version, reporter: &impl Reporter) -> TResult<()> {
        match self.release_index {
            Some(index) if has_release(msrv, index) => Ok(()),
            Some(index) => Err(InvalidMsrvSetError {
                input: msrv.clone(),
                search_space: index.releases(),
            }
            .into()),
            None => {
                reporter.report_event(UnableToConfirmValidReleaseVersion {})?;
                Ok(())
            }
        }
    }
}

fn has_release(configured_msrv: &Version, release_index: &ReleaseIndex) -> bool {
    release_index
        .stable_releases()
        .iter()
        .any(|release| matches_release(configured_msrv, release))
}

fn matches_release(msrv: &Version, release: &RustRelease<Stable>) -> bool {
    let version = release.version().version;

    let major_match = version.major() == msrv.major();
    let minor_match = version.minor() == msrv.minor();
    let patch_match = msrv.patch().is_none_or(|patch| version.patch() == patch);

    major_match && minor_match && patch_match
}

pub(super) fn set_msrv(
    reporter: &impl Reporter,
    cargo_toml: &Utf8Path,
    package: Option<SelectedPackage>,
    msrv: &Version,
) -> TResult<()> {
    write_manifest_msrv(cargo_toml, msrv)?;
    report_set_msrv(reporter, cargo_toml, package, msrv)
}

fn report_set_msrv(
    reporter: &impl Reporter,
    cargo_toml: &Utf8Path,
    package: Option<SelectedPackage>,
    msrv: &Version,
) -> TResult<()> {
    reporter.report_event(AuxiliaryOutput::new(
        Destination::file(cargo_toml.to_path_buf()),
        AuxiliaryOutputItem::msrv(MsrvKind::RustVersion),
    ))?;

    // Report that the MSRV was set
    reporter.report_event(
        SetResult::new(msrv.clone(), cargo_toml.to_path_buf()).with_package(package),
    )?;

    Ok(())
}

#[cfg(test)]
mod valid_release_tests {
    use cargo_msrv_rust_releases::{ReleaseIndex, RustRelease, Stable};
    use std::iter::FromIterator;

    use crate::sub_command::set::has_release;
    use version_number::Version;

    #[test]
    fn releases_include_bare() {
        let bare = Version::new_base_version(1, 55);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 55, 0), None, [])]);

        assert!(has_release(&bare, &index))
    }

    #[test]
    fn releases_does_not_include_bare() {
        let bare = Version::new_base_version(1, 55);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 54, 0), None, [])]);

        assert!(!has_release(&bare, &index))
    }

    #[test]
    fn releases_includes_bare_with_patch() {
        let bare = Version::new_full_version(1, 55, 100);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 55, 100), None, [])]);

        assert!(has_release(&bare, &index))
    }

    #[test]
    fn releases_does_not_include_bare_with_patch() {
        let bare = Version::new_full_version(1, 55, 100);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 55, 0), None, [])]);

        assert!(!has_release(&bare, &index))
    }
}
