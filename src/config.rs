use crate::errors::{CargoMSRVError, TResult};
use clap::ArgMatches;
use rust_releases::semver;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Progress bar rendered to stderr
    Human,
    /// Json status updates printed to stdout
    Json,
    /// No output -- meant to be used for testing
    None,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Human
    }
}

/// Gets a [`Config`] from the given matches, but sets output_format to None
///
/// This is meant to be used for testing
pub fn test_config_from_matches<'a>(matches: &'a ArgMatches<'a>) -> TResult<Config<'a>> {
    let mut config = Config::try_from(matches)?;
    config.output_format = OutputFormat::None;
    Ok(config)
}

#[derive(Debug, Clone, Copy)]
pub enum ModeIntent {
    // Determines the MSRV for a project
    DetermineMSRV,
    // Verifies the given MSRV
    VerifyMSRV,
}

impl From<ModeIntent> for &'static str {
    fn from(action: ModeIntent) -> Self {
        match action {
            ModeIntent::DetermineMSRV => "determine-msrv",
            ModeIntent::VerifyMSRV => "verify-msrv",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ReleaseSource {
    RustChangelog,
    RustDist,
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

#[derive(Debug, Clone)]
pub struct Config<'a> {
    mode_intent: ModeIntent,
    target: String,
    check_command: Vec<&'a str>,
    crate_path: Option<PathBuf>,
    include_all_patch_releases: bool,
    minimum_version: Option<semver::Version>,
    maximum_version: Option<semver::Version>,
    bisect: bool,
    output_toolchain_file: bool,
    ignore_lockfile: bool,
    output_format: OutputFormat,
    release_source: ReleaseSource,
    no_tracing: bool,
    no_read_min_edition: bool,
}

impl<'a> Config<'a> {
    pub fn new(mode_intent: ModeIntent, target: String) -> Self {
        Self {
            mode_intent,
            target,
            check_command: vec!["cargo", "check", "--all"],
            crate_path: None,
            include_all_patch_releases: false,
            minimum_version: None,
            maximum_version: None,
            bisect: false,
            output_toolchain_file: false,
            ignore_lockfile: false,
            output_format: OutputFormat::Human,
            release_source: ReleaseSource::RustChangelog,
            no_tracing: false,
            no_read_min_edition: false,
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

    pub fn bisect(&self) -> bool {
        self.bisect
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

    pub fn no_tracing(&self) -> bool {
        self.no_tracing
    }

    pub fn no_read_min_version(&self) -> bool {
        self.no_read_min_edition
    }
}

#[derive(Debug, Clone)]
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

    pub fn bisect(mut self, answer: bool) -> Self {
        self.inner.bisect = answer;
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

    pub fn no_tracing(mut self, choice: bool) -> Self {
        self.inner.no_tracing = choice;
        self
    }

    pub fn build(self) -> Config<'a> {
        self.inner
    }
}

impl<'config> TryFrom<&'config ArgMatches<'config>> for Config<'config> {
    type Error = CargoMSRVError;

    fn try_from(matches: &'config ArgMatches<'config>) -> Result<Self, Self::Error> {
        use crate::cli::id;
        use crate::fetch::default_target;

        let arg_matches = matches
            .subcommand_matches(id::SUB_COMMAND_MSRV)
            .ok_or(CargoMSRVError::UnableToParseCliArgs)?;

        let action_intent = if arg_matches.is_present(id::ARG_VERIFY) {
            ModeIntent::VerifyMSRV
        } else {
            ModeIntent::DetermineMSRV
        };

        // FIXME: if set, we don't need to do this; in case we can't find it, it may fail here, but atm can't be manually supplied at all
        let target = default_target()?;

        let mut builder = ConfigBuilder::new(action_intent, &target);

        // set the command which will be used to check if a project can build
        let check_cmd = arg_matches.values_of(id::ARG_CUSTOM_CHECK);
        if let Some(cmd) = check_cmd {
            builder = builder.check_command(cmd.collect());
        }

        // set the cargo workspace path
        let crate_path = arg_matches.value_of(id::ARG_SEEK_PATH);
        builder = builder.crate_path(crate_path);

        // set a custom target
        let custom_target = arg_matches.value_of(id::ARG_SEEK_CUSTOM_TARGET);
        if let Some(target) = custom_target {
            builder = builder.target(target);
        }

        if let Some(min) = arg_matches.value_of(id::ARG_MIN) {
            builder = builder.minimum_version(parse_version(min)?)
        }

        if let Some(max) = arg_matches.value_of(id::ARG_MAX) {
            builder = builder.maximum_version(rust_releases::semver::Version::parse(max)?)
        }

        builder = builder.bisect(arg_matches.is_present(id::ARG_BISECT));

        builder = builder
            .include_all_patch_releases(arg_matches.is_present(id::ARG_INCLUDE_ALL_PATCH_RELEASES));

        builder = builder.output_toolchain_file(arg_matches.is_present(id::ARG_TOOLCHAIN_FILE));

        builder = builder.ignore_lockfile(arg_matches.is_present(id::ARG_IGNORE_LOCKFILE));

        let output_format = arg_matches.value_of(id::ARG_OUTPUT_FORMAT);
        if let Some(output_format) = output_format {
            let output_format = match output_format {
                "json" => OutputFormat::Json,
                _ => unreachable!(),
            };

            builder = builder.output_format(output_format);
        }

        let release_source = arg_matches.value_of(id::ARG_RELEASE_SOURCE);
        if let Some(release_source) = release_source {
            let release_source = ReleaseSource::try_from(release_source)?;
            builder = builder.release_source(release_source);
        }

        builder = builder.no_tracing(arg_matches.is_present(id::ARG_NO_LOG));

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
        assert_eq!(version, expected_version)
    }
}
