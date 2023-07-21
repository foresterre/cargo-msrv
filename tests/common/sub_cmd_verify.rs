use crate::common::reporter::EventTestDevice;
use cargo_msrv::cli::CargoCli;
use cargo_msrv::error::CargoMSRVError;
use cargo_msrv::{verify_msrv, Context};
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
    let Context::Verify(verify_opts, find_opts, shared_opts) = ctx else {panic!()};

    // Limit the available versions: this ensures we don't need to incrementally install more toolchains
    //  as more Rust toolchains become available.
    let available_versions = ReleaseIndex::from_iter(releases);

    let device = EventTestDevice::default();

    // Determine the MSRV from the index of available releases.
    verify_msrv(
        device.reporter(),
        verify_opts,
        find_opts,
        shared_opts,
        &available_versions,
    )
}
