use crate::context::SetContext;
use crate::error::InvalidMsrvSetError;
use crate::reporter::Reporter;
use crate::reporter::event::{
    AuxiliaryOutput, AuxiliaryOutputItem, Destination, MsrvKind, SetResult,
    UnableToConfirmValidReleaseVersion,
};
use crate::{SubCommand, TResult};
use cargo_msrv_rust_releases::{ReleaseIndex, RustRelease, Stable};
use cargo_msrv_rust_tools::write_manifest_msrv;
use cargo_msrv_types::BareVersion;

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
        let configured_msrv = &ctx.msrv;

        match self.release_index {
            Some(index) if has_release(configured_msrv, index) => {
                set_msrv(ctx, reporter, configured_msrv)
            }
            Some(index) => Err(InvalidMsrvSetError {
                input: configured_msrv.clone(),
                search_space: index.releases(),
            }
            .into()),
            None => {
                reporter.report_event(UnableToConfirmValidReleaseVersion {})?;
                set_msrv(ctx, reporter, configured_msrv)
            }
        }
    }
}

fn has_release(configured_msrv: &BareVersion, release_index: &ReleaseIndex) -> bool {
    release_index
        .stable_releases()
        .iter()
        .any(|release| matches_release(configured_msrv, release))
}

fn matches_release(msrv: &BareVersion, release: &RustRelease<Stable>) -> bool {
    let version = release.version().version;

    let major_match = version.major() == msrv.major();
    let minor_match = version.minor() == msrv.minor();
    let patch_match = msrv.patch().is_none_or(|patch| version.patch() == patch);

    major_match && minor_match && patch_match
}

fn set_msrv(ctx: &SetContext, reporter: &impl Reporter, msrv: &BareVersion) -> TResult<()> {
    let cargo_toml = ctx.environment.manifest();

    write_manifest_msrv(&cargo_toml, msrv)?;

    reporter.report_event(AuxiliaryOutput::new(
        Destination::file(cargo_toml.clone()),
        AuxiliaryOutputItem::msrv(MsrvKind::RustVersion),
    ))?;

    // Report that the MSRV was set
    reporter.report_event(SetResult::new(msrv.clone(), cargo_toml))?;

    Ok(())
}

#[cfg(test)]
mod valid_release_tests {
    use cargo_msrv_rust_releases::{ReleaseIndex, RustRelease, Stable};
    use std::iter::FromIterator;

    use crate::sub_command::set::has_release;
    use cargo_msrv_types::BareVersion;

    #[test]
    fn releases_include_bare() {
        let bare = BareVersion::TwoComponents(1, 55);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 55, 0), None, [])]);

        assert!(has_release(&bare, &index))
    }

    #[test]
    fn releases_does_not_include_bare() {
        let bare = BareVersion::TwoComponents(1, 55);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 54, 0), None, [])]);

        assert!(!has_release(&bare, &index))
    }

    #[test]
    fn releases_includes_bare_with_patch() {
        let bare = BareVersion::ThreeComponents(1, 55, 100);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 55, 100), None, [])]);

        assert!(has_release(&bare, &index))
    }

    #[test]
    fn releases_does_not_include_bare_with_patch() {
        let bare = BareVersion::ThreeComponents(1, 55, 100);
        let index =
            ReleaseIndex::from_iter(vec![RustRelease::new(Stable::new(1, 55, 0), None, [])]);

        assert!(!has_release(&bare, &index))
    }
}
