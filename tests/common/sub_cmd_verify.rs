use crate::common::reporter::EventTestDevice;
use cargo_msrv::cli::CargoCli;
use cargo_msrv::compatibility::RustupToolchainCheck;
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::{Context, SubCommand, Verify};
use rust_releases::{Release, ReleaseIndex};
use std::convert::TryFrom;
use std::ffi::OsString;
use std::iter::FromIterator;

pub fn run_verify<I, T, S>(with_args: I, releases: S) -> Result<(), CargoMSRVError>
where
    T: Into<OsString> + Clone,
    I: IntoIterator<Item = T>,
    S: IntoIterator<Item = Release>,
{
    let matches = CargoCli::parse_args(with_args);
    let opts = matches.to_cargo_msrv_cli().to_opts();
    let ctx = Context::try_from(opts)?;
    let verify_ctx = ctx.to_verify_context();

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions = ReleaseIndex::from_iter(releases);

    let device = EventTestDevice::default();

    let ignore_toolchain = verify_ctx.ignore_lockfile;
    let no_check_feedback = verify_ctx.no_check_feedback;
    let env = &verify_ctx.environment;

    let runner = RustupToolchainCheck::new(
        device.reporter(),
        ignore_toolchain,
        no_check_feedback,
        env,
        verify_ctx.run_command(),
    );

    // Determine the MSRV from the index of available releases.
    let cmd = Verify::new(&available_versions, runner);

    cmd.run(&verify_ctx, device.reporter())
}
