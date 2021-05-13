use crate::errors::CargoMSRVError;
use clap::ArgMatches;
use rust_releases::semver;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config<'a> {
    target: String,
    check_command: Vec<&'a str>,
    crate_path: Option<PathBuf>,
    include_all_patch_releases: bool,
    minimum_version: Option<semver::Version>,
    maximum_version: Option<semver::Version>,
    bisect: bool,
    output_toolchain_file: bool,
    ignore_lockfile: bool,
}

impl<'a> Config<'a> {
    pub fn new(target: String) -> Self {
        Self {
            target,
            check_command: vec!["cargo", "check", "--all"],
            crate_path: None,
            include_all_patch_releases: false,
            minimum_version: None,
            maximum_version: None,
            bisect: false,
            output_toolchain_file: false,
            ignore_lockfile: false,
        }
    }

    pub fn target(&self) -> &String {
        &self.target
    }

    pub fn check_command(&self) -> &Vec<&'a str> {
        &self.check_command
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
}

#[derive(Debug, Clone)]
pub struct ConfigBuilder<'a> {
    inner: Config<'a>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn new(default_target: &str) -> Self {
        Self {
            inner: Config::new(default_target.to_string()),
        }
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

    pub fn minimum_version(mut self, version: Option<semver::Version>) -> Self {
        self.inner.minimum_version = version;
        self
    }

    pub fn maximum_version(mut self, version: Option<semver::Version>) -> Self {
        self.inner.maximum_version = version;
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

    pub fn build(self) -> Config<'a> {
        self.inner
    }
}

impl<'config> TryFrom<&'config ArgMatches<'config>> for Config<'config> {
    type Error = CargoMSRVError;

    fn try_from(matches: &'config ArgMatches<'config>) -> Result<Self, Self::Error> {
        use crate::cli::id;
        use crate::fetch::default_target;

        let target = default_target()?;

        let arg_matches = matches
            .subcommand_matches(id::SUB_COMMAND_MSRV)
            .ok_or(CargoMSRVError::UnableToParseCliArgs)?;

        let mut builder = ConfigBuilder::new(&target);

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
            builder = builder.minimum_version(Some(rust_releases::semver::Version::parse(min)?))
        }

        if let Some(max) = arg_matches.value_of(id::ARG_MAX) {
            builder = builder.maximum_version(Some(rust_releases::semver::Version::parse(max)?))
        }

        builder = builder.bisect(arg_matches.is_present(id::ARG_BISECT));

        builder = builder
            .include_all_patch_releases(arg_matches.is_present(id::ARG_INCLUDE_ALL_PATCH_RELEASES));

        builder = builder.output_toolchain_file(arg_matches.is_present(id::ARG_TOOLCHAIN_FILE));

        builder = builder.ignore_lockfile(arg_matches.is_present(id::ARG_IGNORE_LOCKFILE));

        Ok(builder.build())
    }
}
