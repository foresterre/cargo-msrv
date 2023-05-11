use clap::ValueEnum;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::cli::CargoCli;
use crate::config::list::ListCmdConfig;
use crate::config::set::SetCmdConfig;
use crate::config::verify::VerifyCmdConfig;
use crate::ctx::{ContextValues, LazyContext};
use rust_releases::semver;

use crate::error::{CargoMSRVError, TResult};
use crate::log_level::LogLevel;
use crate::manifest::bare_version;

pub(crate) mod list;
pub(crate) mod set;
pub(crate) mod verify;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    /// Progress bar rendered to stderr
    #[default]
    Human,
    /// Json status updates printed to stdout
    Json,
    /// Minimal output, usually just the result, such as the MSRV or whether verify succeeded or failed
    Minimal,
    /// No output -- meant to be used for debugging and testing
    #[value(skip)]
    None,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Json => write!(f, "json"),
            Self::Minimal => write!(f, "minimal"),
            Self::None => write!(f, "none"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            "minimal" => Ok(Self::Minimal),
            unknown => Err(CargoMSRVError::InvalidConfig(format!(
                "Given output format '{}' is not valid",
                unknown
            ))),
        }
    }
}

/// Gets a [`Config`] from the given matches, but sets output_format to None
///
/// This is meant to be used for testing
pub fn test_config_from_cli(cli: &CargoCli) -> TResult<Config> {
    let mut config = Config::try_from(cli)?;
    config.output_format = OutputFormat::None;
    Ok(config)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubcommandId {
    // Determines the MSRV for a project
    Find,
    // List the MSRV's as specified by package authors
    List,
    // Verifies the given MSRV
    Verify,
    // Set the MSRV in the Cargo manifest to a given value
    Set,
    // Shows the MSRV of the current crate as specified in the Cargo manifest
    Show,
    // Forward the subcommand to cargo
    Forward(ForwardedSubcommandId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForwardedSubcommandId {
    // Build the project with the given MSRV
    Build,
    // Check the project with the given MSRV
    Check,
    // Test the project with the given MSRV
    Test,
}

impl From<SubcommandId> for &'static str {
    fn from(id: SubcommandId) -> Self {
        match id {
            SubcommandId::Find => "find",
            SubcommandId::List => "list",
            SubcommandId::Verify => "verify",
            SubcommandId::Set => "set",
            SubcommandId::Show => "show",
            SubcommandId::Forward(fwd) => match fwd {
                ForwardedSubcommandId::Build => "build",
                ForwardedSubcommandId::Check => "check",
                ForwardedSubcommandId::Test => "test",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseSource {
    #[default]
    RustChangelog,
    #[cfg(feature = "rust-releases-dist-source")]
    RustDist,
}

impl FromStr for ReleaseSource {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl From<ReleaseSource> for &'static str {
    fn from(value: ReleaseSource) -> Self {
        match value {
            ReleaseSource::RustChangelog => "rust-changelog",
            #[cfg(feature = "rust-releases-dist-source")]
            ReleaseSource::RustDist => "rust-dist",
        }
    }
}

impl TryFrom<&str> for ReleaseSource {
    type Error = CargoMSRVError;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
        match source {
            "rust-changelog" => Ok(Self::RustChangelog),
            #[cfg(feature = "rust-releases-dist-source")]
            "rust-dist" => Ok(Self::RustDist),
            s => Err(CargoMSRVError::RustReleasesSourceParseError(s.to_string())),
        }
    }
}

impl fmt::Display for ReleaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustChangelog => write!(f, "rust-changelog"),
            #[cfg(feature = "rust-releases-dist-source")]
            Self::RustDist => write!(f, "rust-dist"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMethod {
    Linear,
    Bisect,
}

impl From<SearchMethod> for &'static str {
    fn from(method: SearchMethod) -> Self {
        match method {
            SearchMethod::Linear => "linear",
            SearchMethod::Bisect => "bisect",
        }
    }
}

impl Default for SearchMethod {
    fn default() -> Self {
        Self::Bisect
    }
}

// TODO{foresterre}:
//  This Config approach does not scale with the amount of options
//  we now have. It also not allow us to easily merge several layers of option inputs,
//  for example from the CLI, from env vars, or from a configuration file.
#[derive(Debug, Clone)]
pub struct Config<'a> {
    subcommand_id: SubcommandId,
    target: String,
    check_command: Vec<&'a str>,
    crate_path: Option<PathBuf>,
    manifest_path: Option<PathBuf>,
    include_all_patch_releases: bool,
    minimum_version: Option<bare_version::BareVersion>,
    maximum_version: Option<bare_version::BareVersion>,
    search_method: SearchMethod,
    output_toolchain_file: bool,
    write_msrv: bool,
    ignore_lockfile: bool,
    output_format: OutputFormat,
    release_source: ReleaseSource,
    tracing_config: Option<TracingOptions>,
    no_read_min_edition: Option<semver::Version>,
    no_check_feedback: bool,

    sub_command_config: SubCommandConfig,
    ctx: LazyContext,
}

impl<'a> Config<'a> {
    pub fn new<T: Into<String>>(subcommand_id: SubcommandId, target: T) -> Self {
        Self {
            subcommand_id,
            target: target.into(),
            check_command: vec!["cargo", "check"],
            crate_path: None,
            manifest_path: None,
            include_all_patch_releases: false,
            minimum_version: None,
            maximum_version: None,
            search_method: SearchMethod::default(),
            output_toolchain_file: false,
            write_msrv: false,
            ignore_lockfile: false,
            output_format: OutputFormat::Human,
            release_source: ReleaseSource::RustChangelog,
            tracing_config: None,
            no_read_min_edition: None,
            no_check_feedback: false,
            sub_command_config: SubCommandConfig::None,
            ctx: LazyContext::default(),
        }
    }

    pub fn subcommand_id(&self) -> SubcommandId {
        self.subcommand_id
    }

    pub fn target(&self) -> &String {
        &self.target
    }

    pub fn check_command(&self) -> &Vec<&'a str> {
        &self.check_command
    }

    pub fn check_command_string(&self) -> String {
        self.check_command.join(" ")
    }

    /// Should not be used directly. Use the context instead.
    pub fn crate_path(&self) -> Option<&Path> {
        self.crate_path.as_deref()
    }

    /// Should not be used directly. Use the context instead.
    pub fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }

    pub fn include_all_patch_releases(&self) -> bool {
        self.include_all_patch_releases
    }

    pub fn minimum_version(&self) -> Option<&bare_version::BareVersion> {
        self.minimum_version.as_ref()
    }

    pub fn maximum_version(&self) -> Option<&bare_version::BareVersion> {
        self.maximum_version.as_ref()
    }

    pub fn search_method(&self) -> SearchMethod {
        self.search_method
    }

    pub fn output_toolchain_file(&self) -> bool {
        self.output_toolchain_file
    }

    pub fn write_msrv(&self) -> bool {
        self.write_msrv
    }

    pub fn ignore_lockfile(&self) -> bool {
        self.ignore_lockfile
    }

    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    pub fn release_source(&self) -> ReleaseSource {
        self.release_source
    }

    /// Options as to configure tracing (and logging) settings. If absent, tracing will be disabled.
    pub fn tracing(&self) -> Option<&TracingOptions> {
        self.tracing_config.as_ref()
    }

    pub fn no_read_min_version(&self) -> Option<&semver::Version> {
        self.no_read_min_edition.as_ref()
    }

    pub fn no_check_feedback(&self) -> bool {
        self.no_check_feedback
    }

    pub fn sub_command_config(&self) -> &SubCommandConfig {
        &self.sub_command_config
    }

    pub fn context(&self) -> &LazyContext {
        &self.ctx
    }

    fn init_context(mut self) -> Self {
        let values = ContextValues::from_config(&self);
        self.ctx.init(values);
        self
    }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct ConfigBuilder<'a> {
    inner: Config<'a>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new(subcommand_id: SubcommandId, default_target: &str) -> Self {
        Self {
            inner: Config::new(subcommand_id, default_target.to_string()),
        }
    }

    pub fn from_config(config: &'a Config) -> Self {
        Self {
            inner: config.clone(),
        }
    }

    pub fn mode_intent(mut self, subcommand_id: SubcommandId) -> Self {
        self.inner.subcommand_id = subcommand_id;
        self
    }

    pub fn target(mut self, target: &str) -> Self {
        self.inner.target = target.to_string();
        self
    }

    pub fn check_command(mut self, cmd: Vec<&'a str>) -> Self {
        self.inner.check_command = cmd;
        self
    }

    pub fn crate_path<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.inner.crate_path = path.map(|p| PathBuf::from(p.as_ref()));
        self
    }

    pub fn manifest_path<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.inner.manifest_path = path.map(|p| PathBuf::from(p.as_ref()));
        self
    }

    pub fn get_crate_path(&self) -> Option<&Path> {
        self.inner.crate_path.as_deref()
    }

    pub fn include_all_patch_releases(mut self, answer: bool) -> Self {
        self.inner.include_all_patch_releases = answer;
        self
    }

    pub fn minimum_version(mut self, version: bare_version::BareVersion) -> Self {
        self.inner.minimum_version = Some(version);
        self
    }

    pub fn maximum_version(mut self, version: bare_version::BareVersion) -> Self {
        self.inner.maximum_version = Some(version);
        self
    }

    pub fn search_method(mut self, method: SearchMethod) -> Self {
        self.inner.search_method = method;
        self
    }

    pub fn output_toolchain_file(mut self, choice: bool) -> Self {
        self.inner.output_toolchain_file = choice;
        self
    }

    pub fn write_msrv(mut self, choice: bool) -> Self {
        self.inner.write_msrv = choice;
        self
    }

    pub fn ignore_lockfile(mut self, choice: bool) -> Self {
        self.inner.ignore_lockfile = choice;
        self
    }

    pub fn output_format(mut self, output_format: OutputFormat) -> Self {
        self.inner.output_format = output_format;
        self
    }

    pub fn release_source(mut self, release_source: ReleaseSource) -> Self {
        self.inner.release_source = release_source;
        self
    }

    pub fn tracing_config(mut self, cfg: TracingOptions) -> Self {
        self.inner.tracing_config = Some(cfg);
        self
    }

    pub fn no_read_min_edition(mut self, version: semver::Version) -> Self {
        self.inner.no_read_min_edition = Some(version);
        self
    }

    pub fn no_check_feedback(mut self, choice: bool) -> Self {
        self.inner.no_check_feedback = choice;
        self
    }

    pub fn sub_command_config(mut self, cmd_config: SubCommandConfig) -> Self {
        self.inner.sub_command_config = cmd_config;
        self
    }

    pub fn build(self) -> Config<'a> {
        self.inner.init_context()
    }
}

macro_rules! as_sub_command_config {
    ($subcmd:ident, $variant:ident, $out_type:ty) => {
        pub(crate) fn $subcmd(&self) -> &$out_type {
            if let Self::$variant(c) = self {
                c
            } else {
                // In this case we made a programming error
                unreachable!()
            }
        }
    };
}

#[derive(Debug, Clone)]
pub enum SubCommandConfig {
    None,
    ListConfig(ListCmdConfig),
    SetConfig(SetCmdConfig),
    ShowConfig,
    VerifyConfig(VerifyCmdConfig),
}

impl SubCommandConfig {
    as_sub_command_config!(list, ListConfig, ListCmdConfig);
    as_sub_command_config!(set, SetConfig, SetCmdConfig);
    as_sub_command_config!(verify, VerifyConfig, VerifyCmdConfig);
}

#[derive(Debug, Clone)]
pub struct TracingOptions {
    target: TracingTargetOption,
    level: LogLevel,
}

impl TracingOptions {
    pub fn new(target: TracingTargetOption, level: LogLevel) -> Self {
        Self { target, level }
    }
}

impl Default for TracingOptions {
    fn default() -> Self {
        Self {
            target: TracingTargetOption::File,
            level: LogLevel::default(),
        }
    }
}

impl TracingOptions {
    pub fn target(&self) -> &TracingTargetOption {
        &self.target
    }

    pub fn level(&self) -> &LogLevel {
        &self.level
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum TracingTargetOption {
    File,
    Stdout,
}

impl Default for TracingTargetOption {
    fn default() -> Self {
        Self::File
    }
}

impl TracingTargetOption {
    pub const FILE: &'static str = "file";
    pub const STDOUT: &'static str = "stdout";
}

impl FromStr for TracingTargetOption {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::FILE => Ok(Self::File),
            Self::STDOUT => Ok(Self::Stdout),
            unknown => Err(CargoMSRVError::InvalidConfig(format!(
                "Given log target '{}' is not valid",
                unknown
            ))),
        }
    }
}
