use clap::ArgEnum;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::cli::CargoCli;
use crate::config::list::ListCmdConfig;
use crate::config::set::SetCmdConfig;
use rust_releases::semver;

use crate::errors::{CargoMSRVError, TResult};
use crate::log_level::LogLevel;
use crate::manifest::bare_version;

pub(crate) mod list;
pub(crate) mod set;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Progress bar rendered to stderr
    Human,
    /// Json status updates printed to stdout
    Json,
    /// No output -- meant to be used for debugging and testing
    None,
    /// Save all versions tested and save success result for all runs -- meant to be used for testing
    TestSuccesses,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Human
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Json => write!(f, "json"),
            Self::None => write!(f, "none"),
            Self::TestSuccesses => write!(f, "test-successes"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = CargoMSRVError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            unknown => Err(CargoMSRVError::InvalidConfig(format!(
                "Given output format '{}' is not valid",
                unknown
            ))),
        }
    }
}

impl OutputFormat {
    pub const JSON: &'static str = "json";

    /// A set of formats which may be given as a configuration option
    ///   through the CLI.
    pub fn custom_formats() -> &'static [&'static str] {
        &["human", Self::JSON]
    }

    /// Parse the output format from the given `&str`.
    ///
    /// **Panics**
    ///
    /// Panics if the format is not known, or can not be set by a user.
    pub fn from_custom_format_str(item: &str) -> Self {
        match item {
            Self::JSON => Self::Json,
            _ => unreachable!(),
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

#[derive(Debug, Clone, Copy)]
pub enum ModeIntent {
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
}

impl From<ModeIntent> for &'static str {
    fn from(action: ModeIntent) -> Self {
        match action {
            ModeIntent::Find => "determine-msrv",
            ModeIntent::List => "list-msrv",
            ModeIntent::Verify => "verify-msrv",
            ModeIntent::Set => "set-msrv",
            ModeIntent::Show => "show-msrv",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReleaseSource {
    RustChangelog,
    #[cfg(feature = "rust-releases-dist-source")]
    RustDist,
}

impl Default for ReleaseSource {
    fn default() -> Self {
        Self::RustChangelog
    }
}

impl ReleaseSource {
    pub(crate) fn variants() -> &'static [&'static str] {
        &[
            "rust-changelog",
            #[cfg(feature = "rust-releases-dist-source")]
            "rust-dist",
        ]
    }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustChangelog => write!(f, "rust-changelog"),
            #[cfg(feature = "rust-releases-dist-source")]
            Self::RustDist => write!(f, "rust-dist"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    mode_intent: ModeIntent,
    target: String,
    check_command: Vec<&'a str>,
    crate_path: Option<PathBuf>,
    include_all_patch_releases: bool,
    minimum_version: Option<bare_version::BareVersion>,
    maximum_version: Option<bare_version::BareVersion>,
    search_method: SearchMethod,
    output_toolchain_file: bool,
    ignore_lockfile: bool,
    output_format: OutputFormat,
    release_source: ReleaseSource,
    tracing_config: Option<TracingOptions>,
    no_read_min_edition: Option<semver::Version>,
    no_check_feedback: bool,

    sub_command_config: SubCommandConfig,
}

impl<'a> Config<'a> {
    pub fn new(mode_intent: ModeIntent, target: String) -> Self {
        Self {
            mode_intent,
            target,
            check_command: vec!["cargo", "check"],
            crate_path: None,
            include_all_patch_releases: false,
            minimum_version: None,
            maximum_version: None,
            search_method: SearchMethod::default(),
            output_toolchain_file: false,
            ignore_lockfile: false,
            output_format: OutputFormat::Human,
            release_source: ReleaseSource::RustChangelog,
            tracing_config: None,
            no_read_min_edition: None,
            no_check_feedback: false,
            sub_command_config: SubCommandConfig::None,
        }
    }

    pub fn action_intent(&self) -> ModeIntent {
        self.mode_intent
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

    pub fn crate_path(&self) -> Option<&Path> {
        self.crate_path.as_deref()
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
}

#[derive(Debug, Clone)]
#[must_use]
pub struct ConfigBuilder<'a> {
    inner: Config<'a>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new(action_intent: ModeIntent, default_target: &str) -> Self {
        Self {
            inner: Config::new(action_intent, default_target.to_string()),
        }
    }

    pub fn mode_intent(mut self, mode_intent: ModeIntent) -> Self {
        self.inner.mode_intent = mode_intent;
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
        self.inner
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
}

impl SubCommandConfig {
    as_sub_command_config!(list, ListConfig, ListCmdConfig);
    as_sub_command_config!(set, SetConfig, SetCmdConfig);
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

#[derive(Debug, Copy, Clone, ArgEnum)]
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
