use rust_releases::semver;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CmdMatches<'a> {
    target: String,
    check_command: Vec<&'a str>,
    crate_path: Option<PathBuf>,
    include_all_patch_releases: bool,
    minimum_version: Option<semver::Version>,
    maximum_version: Option<semver::Version>,
    output_toolchain_file: bool,
}

impl<'a> CmdMatches<'a> {
    pub fn new(target: String) -> Self {
        Self {
            target,
            check_command: vec!["cargo", "check", "--all"],
            crate_path: None,
            include_all_patch_releases: false,
            minimum_version: None,
            maximum_version: None,
            output_toolchain_file: false,
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

    pub fn output_toolchain_file(&self) -> bool {
        self.output_toolchain_file
    }
}

#[derive(Debug, Clone)]
pub struct CmdMatchesBuilder<'a> {
    inner: CmdMatches<'a>,
}

impl<'a> CmdMatchesBuilder<'a> {
    pub fn new(default_target: &str) -> Self {
        Self {
            inner: CmdMatches::new(default_target.to_string()),
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

    pub fn output_toolchain_file(mut self, choice: bool) -> Self {
        self.inner.output_toolchain_file = choice;
        self
    }

    pub fn build(self) -> CmdMatches<'a> {
        self.inner
    }
}
