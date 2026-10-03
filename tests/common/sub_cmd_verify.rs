use crate::common::reporter::EventTestDevice;
use cargo_msrv::cli::CargoCli;
use cargo_msrv::compatibility::{CheckTarget, RustupToolchainCheck};
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::{Context, SubCommand, Verify};
use cargo_msrv_rust_releases::{ReleaseIndex, RustRelease, Stable};
use std::convert::TryFrom;
use std::ffi::OsString;
use std::iter::FromIterator;

pub fn run_verify<I, T, S>(with_args: I, releases: S) -> Result<(), CargoMSRVError>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
    S: IntoIterator<Item = RustRelease<Stable>>,
{
    let matches = CargoCli::parse_args(with_args);
    let opts = matches.to_cargo_msrv_cli().to_opts();
    let ctx = Context::try_from(opts)?;
    let verify_ctx = ctx.to_verify_context().unwrap();

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions = ReleaseIndex::from_iter(releases);

    let device = EventTestDevice::default();

    let check_for = |target: CheckTarget| {
        RustupToolchainCheck::new(
            device.reporter(),
            verify_ctx.ignore_lockfile,
            verify_ctx.no_check_feedback,
            false, /* Marking unavailable versions as incompatible, which is always false for `verify`  */
            verify_ctx.environment.lock(),
            target,
        )
    };

    // Determine the MSRV from the index of available releases.
    let cmd = Verify::new(&available_versions, check_for);

    cmd.run(&verify_ctx, device.reporter())
}
