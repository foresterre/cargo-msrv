use crate::config::{Config, ModeIntent};
use crate::errors::{CargoMSRVError, IoErrorSource, TResult};
use crate::manifest::bare_version::BareVersion;
use crate::manifest::{CargoManifest, CargoManifestParser, TomlParser};
use crate::paths::crate_root_folder;
use crate::reporter::Output;
use crate::semver::Version;
use crate::toolchain::ToolchainSpec;
use crate::SubCommand;
use rust_releases::semver;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml_edit::Document;

#[derive(Default)]
pub struct Doctor;

impl SubCommand for Doctor {
    fn run<R: Output>(&self, config: &Config, reporter: &R) -> TResult<()> {
        run_doctor(config, reporter)
    }
}

fn run_doctor<R: Output>(config: &Config, output: &R) -> TResult<()> {
    let crate_folder = crate_root_folder(config)?;
    let cargo_toml = crate_folder.join("Cargo.toml");

    let contents = std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
        error,
        source: IoErrorSource::ReadFile(cargo_toml),
    })?;

    let manifest = CargoManifestParser::default().parse::<Document>(&contents)?;
    let manifest = CargoManifest::try_from(manifest)?;

    let msrv = manifest.minimum_rust_version();
    if msrv.is_some() {
        output.finish_success(
            ModeIntent::Show,
            msrv.map(BareVersion::to_semver_version).as_ref(),
        );
    } else {
        output.finish_failure(ModeIntent::Show, None);
    }

    Ok(())
}

#[derive(Debug)]
enum DoctorChecks {
    InSyncMsrv(InSyncMsrv),
}

#[derive(Debug)]
struct InSyncMsrv {
    locations: RustVersionLocations,
}

#[derive(Debug)]
struct RustVersionLocations {
    sources: Vec<Location>,
}

impl RustVersionLocations {}

trait RustVersion {
    fn read_version(&self) -> TResult<Option<semver::Version>>;
}

enum Location {
    CargoManifest(CargoManifest),
    ToolchainFile(ToolchainFile),
    RustFmtFile(toml_edit::Document),
    ClippyFile(toml_edit::Document),
}

impl RustVersion for Location {
    fn read_version(&self) -> TResult<Option<Version>> {
        match self {
            Location::CargoManifest(manifest) => manifest.read_version(),
            Location::ToolchainFile(file) => file.read_version(),
            Location::RustFmtFile(file) => file.read_version(),
            Location::ClippyFile(file) => file.read_version(),
        }
    }
}

impl RustVersion for CargoManifest {
    fn read_version(&self) -> TResult<Option<Version>> {
        Ok(self.minimum_rust_version().map(|v| v.to_semver_version()))
    }
}

impl RustVersion for ToolchainFile {
    fn read_version(&self) -> TResult<Option<Version>> {
        todo!()
    }
}

struct RustFmtFile {
    path: PathBuf,
}

impl RustVersion for RustFmtFile {
    fn read_version(&self) -> TResult<Option<Version>> {
        todo!()
    }
}

struct ClippyFile {
    path: PathBuf,
}

impl RustVersion for ClippyFile {
    fn read_version(&self) -> TResult<Option<Version>> {
        todo!()
    }
}
