use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use toml_edit::{Document, Item};

use crate::config::list::ListCmdConfig;
use crate::config::set::SetCmdConfig;
use clap::ArgMatches;
use rust_releases::semver;

use crate::errors::{CargoMSRVError, IoErrorSource, TResult};

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

impl OutputFormat {
    pub const JSON: &'static str = "json";

    /// A set of formats which may be given as a configuration option
    ///   through the CLI.
    pub fn custom_formats() -> &'static [&'static str] {
        &[Self::JSON]
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
pub fn test_config_from_matches(matches: &ArgMatches) -> TResult<Config> {
    let mut config = Config::try_from(matches)?;
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
    RustDist,
}

impl From<ReleaseSource> for &'static str {
    fn from(value: ReleaseSource) -> Self {
        match value {
            ReleaseSource::RustChangelog => "rust-changelog",
            ReleaseSource::RustDist => "rust-dist",
        }
    }
}

impl TryFrom<&str> for ReleaseSource {
    type Error = CargoMSRVError;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
        match source {
            "rust-changelog" => Ok(Self::RustChangelog),
            "rust-dist" => Ok(Self::RustDist),
            s => Err(CargoMSRVError::RustReleasesSourceParseError(s.to_string())),
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

#[derive(Debug, Clone)]
pub struct Config<'a> {
    mode_intent: ModeIntent,
    target: String,
    check_command: Vec<&'a str>,
    crate_path: Option<PathBuf>,
    include_all_patch_releases: bool,
    minimum_version: Option<semver::Version>,
    maximum_version: Option<semver::Version>,
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

    pub fn minimum_version(&self) -> Option<&semver::Version> {
        self.minimum_version.as_ref()
    }

    pub fn maximum_version(&self) -> Option<&semver::Version> {
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

    pub fn include_all_patch_releases(mut self, answer: bool) -> Self {
        self.inner.include_all_patch_releases = answer;
        self
    }

    pub fn minimum_version(mut self, version: semver::Version) -> Self {
        self.inner.minimum_version = Some(version);
        self
    }

    pub fn maximum_version(mut self, version: semver::Version) -> Self {
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

impl<'config> TryFrom<&'config ArgMatches> for Config<'config> {
    type Error = CargoMSRVError;

    fn try_from(matches: &'config ArgMatches) -> Result<Self, Self::Error> {
        use crate::cli::id;
        use crate::fetch::default_target;

        let action_intent = if matches.subcommand_matches(id::SUB_COMMAND_LIST).is_some() {
            ModeIntent::List
        } else if matches.subcommand_matches(id::SUB_COMMAND_SHOW).is_some() {
            ModeIntent::Show
        } else if matches.subcommand_matches(id::SUB_COMMAND_SET).is_some() {
            ModeIntent::Set
        } else if matches.subcommand_matches(id::SUB_COMMAND_VERIFY).is_some()
            || matches.is_present(id::ARG_VERIFY)
        {
            ModeIntent::Verify
        } else {
            ModeIntent::Find
        };

        // FIXME: if set, we don't need to do this; in case we can't find it, it may fail here, but atm can't be manually supplied at all
        let target = default_target()?;

        let mut builder = ConfigBuilder::new(action_intent, &target);

        // set the command which will be used to check if a project can build
        builder = match matches.subcommand_matches(id::SUB_COMMAND_VERIFY) {
            Some(verify_cmd) => set_custom_check_command(verify_cmd, builder),
            None => set_custom_check_command(matches, builder),
        };

        // set the cargo workspace path
        let crate_path = matches.value_of(id::ARG_SEEK_PATH);
        builder = builder.crate_path(crate_path);

        // set a custom target
        let custom_target = matches.value_of(id::ARG_SEEK_CUSTOM_TARGET);
        if let Some(target) = custom_target {
            builder = builder.target(target);
        }

        match matches.value_of(id::ARG_MIN) {
            Some(min) => builder = builder.minimum_version(parse_version(min)?),
            None if matches.is_present(id::ARG_NO_READ_MIN_EDITION) => {}
            None => {
                let crate_folder = if let Some(ref path) = builder.inner.crate_path {
                    Ok(path.clone())
                } else {
                    std::env::current_dir().map_err(|error| CargoMSRVError::Io {
                        error,
                        source: IoErrorSource::CurrentDir,
                    })
                }?;
                let cargo_toml = crate_folder.join("Cargo.toml");

                let contents =
                    std::fs::read_to_string(&cargo_toml).map_err(|error| CargoMSRVError::Io {
                        error,
                        source: IoErrorSource::ReadFile(cargo_toml.clone()),
                    })?;
                let document = contents
                    .parse::<Document>()
                    .map_err(CargoMSRVError::ParseToml)?;

                if let Some(edition) = document
                    .as_table()
                    .get("package")
                    .and_then(Item::as_table)
                    .and_then(|package_table| package_table.get("edition"))
                    .and_then(Item::as_str)
                {
                    builder = builder.minimum_version(parse_version(edition)?);
                }
            }
        }

        if let Some(max) = matches.value_of(id::ARG_MAX) {
            builder = builder.maximum_version(rust_releases::semver::Version::parse(max)?);
        }

        builder = match (
            matches.is_present(id::ARG_LINEAR),
            matches.is_present(id::ARG_BISECT),
        ) {
            (true, false) => builder.search_method(SearchMethod::Linear),
            (false, true) => builder.search_method(SearchMethod::Bisect),
            _ => builder.search_method(SearchMethod::default()),
        };

        builder = builder
            .include_all_patch_releases(matches.is_present(id::ARG_INCLUDE_ALL_PATCH_RELEASES));

        builder = builder.output_toolchain_file(matches.is_present(id::ARG_TOOLCHAIN_FILE));

        builder = builder.ignore_lockfile(matches.is_present(id::ARG_IGNORE_LOCKFILE));

        if matches.is_present(id::ARG_NO_USER_OUTPUT) {
            builder = builder.output_format(OutputFormat::None);
        } else if let Some(output_format) = matches.value_of(id::ARG_OUTPUT_FORMAT) {
            let output_format = OutputFormat::from_custom_format_str(output_format);
            builder = builder.output_format(output_format);
        }

        let release_source = matches.value_of(id::ARG_RELEASE_SOURCE);
        if let Some(release_source) = release_source {
            let release_source = ReleaseSource::try_from(release_source)?;
            builder = builder.release_source(release_source);
        }

        //
        if !matches.is_present(id::ARG_NO_LOG) {
            let mut config = TracingOptions::default();

            if let Some(log_target) = matches.value_of(id::ARG_LOG_TARGET) {
                config.target = TracingTargetOption::from_str(log_target);
            }

            if let Some(level) = matches.value_of(id::ARG_LOG_LEVEL) {
                config.level = parse_log_level(level);
            }

            builder = builder.tracing_config(config);
        }

        builder = builder.no_check_feedback(matches.is_present(id::ARG_NO_CHECK_FEEDBACK));

        if let Some(cmd) = matches.subcommand_matches(id::SUB_COMMAND_LIST) {
            let cmd_config = ListCmdConfig::try_from(cmd)?;
            builder = builder.sub_command_config(SubCommandConfig::ListConfig(cmd_config));
        } else if let Some(cmd) = matches.subcommand_matches(id::SUB_COMMAND_SET) {
            let cmd_config = SetCmdConfig::try_from(cmd)?;
            builder = builder.sub_command_config(SubCommandConfig::SetConfig(cmd_config));
        }

        Ok(builder.build())
    }
}

fn parse_version(input: &str) -> Result<semver::Version, semver::Error> {
    match input {
        "2015" => Ok(semver::Version::new(1, 0, 0)),
        "2018" => Ok(semver::Version::new(1, 31, 0)),
        "2021" => Ok(semver::Version::new(1, 56, 0)),
        s => semver::Version::parse(s),
    }
}

fn parse_log_level(input: &str) -> tracing::Level {
    input.parse().unwrap_or(tracing::Level::INFO)
}

fn set_custom_check_command<'a>(
    matches: &'a ArgMatches,
    builder: ConfigBuilder<'a>,
) -> ConfigBuilder<'a> {
    use crate::cli::id;

    let check_cmd = matches.values_of(id::ARG_CUSTOM_CHECK);

    if let Some(custom_cmd) = check_cmd {
        builder.check_command(custom_cmd.collect())
    } else {
        builder
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
    level: tracing::Level,
}

impl Default for TracingOptions {
    fn default() -> Self {
        Self {
            target: TracingTargetOption::File,
            level: tracing::Level::INFO,
        }
    }
}

impl TracingOptions {
    pub fn target(&self) -> &TracingTargetOption {
        &self.target
    }

    pub fn level(&self) -> &tracing::Level {
        &self.level
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TracingTargetOption {
    File,
    Stdout,
}

impl TracingTargetOption {
    pub const FILE: &'static str = "file";
    pub const STDOUT: &'static str = "stdout";

    /// Parse the tracing target option from a string.
    ///
    /// NB: Panics if not a valid input
    fn from_str(input: &str) -> Self {
        match input {
            Self::FILE => Self::File,
            Self::STDOUT => Self::Stdout,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use parameterized::parameterized;
    use rust_releases::semver::Version;

    #[parameterized(
        input = {
            "1.35.0",
            "2015",
            "2018",
            "2021",
        },
        expected_version = {
            Version::new(1,35,0),
            Version::new(1,0,0),
            Version::new(1,31,0),
            Version::new(1,56,0),
        }
    )]
    fn parse_version(input: &str, expected_version: Version) {
        let version = super::super::parse_version(input).unwrap();
        assert_eq!(version, expected_version);
    }
}
