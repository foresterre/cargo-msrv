use rust_releases::semver;

#[derive(Debug)]
pub struct ToolchainSpec<'v> {
    version: &'v semver::Version,
    spec: String,
}

impl<'v> ToolchainSpec<'v> {
    pub fn new(target: &str, version: &'v semver::Version) -> Self {
        Self {
            spec: format!("{}-{}", version, target),
            version,
        }
    }

    pub fn spec(&self) -> &str {
        &self.spec
    }

    pub fn version(&self) -> &semver::Version {
        self.version
    }
}
